use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2, PasswordHash, PasswordVerifier,
};
use chrono::{Duration, Local};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use poem::Request;
use poem_openapi::{auth::Bearer, SecurityScheme};
use redis::ConnectionLike;
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

use crate::{model::user::User, repository::user::get_user_by_id, settings::Config};

use super::session::get_session;

/// password hashing
pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);

    // Argon2 with default params (Argon2id v19)
    let argon2 = Argon2::default();

    // Hash password to PHC string ($argon2id$v=19$...)
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)?
        .to_string();

    Ok(password_hash)
}

/// password hash verification
pub fn verify_hash_password(
    password: &str,
    password_hash: &str,
) -> Result<bool, argon2::password_hash::Error> {
    let parsed_hash = PasswordHash::new(password_hash)?;
    let verify = Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok();
    Ok(verify)
}

#[cfg(test)]
mod test_hash_password {
    use super::*;

    #[test]
    fn test_hashing_password() {
        let password = "secretpassword";
        let hash = hash_password(&password);
        assert!(hash.is_ok());
        let hash = hash.unwrap();
        let verify = verify_hash_password(&password, &hash);
        assert!(verify.is_ok());
        assert!(verify.unwrap());
        let verify_false = verify_hash_password("wrongpassword", &hash);
        assert!(verify_false.is_ok());
        assert_eq!(verify_false.unwrap(), false);
    }
}

pub struct Keys {
    pub encoding: EncodingKey,
    pub decoding: DecodingKey,
}

impl Keys {
    fn new(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub id: String,
    pub user_name: String,
    pub exp: i64,
}

impl Claims {
    pub fn new(user_id: &str, user_name: &str, config: Config) -> Self {
        let exp = (Local::now() + Duration::minutes(config.jwt_exp as i64)).timestamp();

        Self {
            id: user_id.to_string(),
            user_name: user_name.to_string(),
            exp,
        }
    }
}

/// Generate token
pub fn encode_token(claims: &Claims, jwt_secret: String) -> anyhow::Result<String> {
    let keys = Keys::new(jwt_secret.as_bytes());
    let token = encode(&Header::default(), &claims, &keys.encoding)?;
    Ok(token)
}

/// Extract payload and Validate token
pub fn decode_token(token: &str, jwt_secret: String) -> anyhow::Result<Claims> {
    let keys = Keys::new(jwt_secret.as_bytes());
    let token_data = decode::<Claims>(token, &keys.decoding, &Validation::default())?;
    Ok(token_data.claims)
}

pub async fn generate_token_from_user(user: User, config: Config) -> anyhow::Result<String> {
    let claims = Claims::new(
        user.id.to_string().as_str(),
        user.user_name.as_str(),
        config.clone(),
    );
    let token = encode_token(&claims, config.jwt_secret)?;
    Ok(token)
}

pub async fn get_user_from_token<C: ConnectionLike>(
    tx: &mut Transaction<'_, Postgres>,
    redis_conn: &mut C,
    jwt_token: Option<String>,
) -> anyhow::Result<Option<User>> {
    if jwt_token.is_none() {
        return Ok(None);
    }
    let session = get_session(redis_conn, jwt_token.unwrap())?;
    if session.is_none() {
        return Ok(None);
    }
    let user_id = Uuid::parse_str(&session.unwrap().user_id)?;
    let (user, _) = get_user_by_id(tx, &user_id).await?;
    Ok(user)
}

#[cfg(test)]
mod test_generate_token {
    use chrono::Local;
    use sqlx::PgPool;
    use uuid::Uuid;

    use crate::{
        core::{
            security::{generate_token_from_user, get_user_from_token, hash_password},
            session::add_session,
        },
        model::{user::User, user_profile::UserProfile},
        settings::get_config,
    };

    #[sqlx::test]
    async fn test_generate_token(pool: PgPool) -> anyhow::Result<()> {
        // Given
        let config = get_config();
        let client = redis::Client::open(config.redis_url.clone()).unwrap();
        let redis_pool = r2d2::Pool::builder().build(client).unwrap();
        let mut redis_conn = redis_pool.get()?;
        let mut tx = pool.begin().await?;
        // Prepare user
        let username = "hello".to_string();
        let password = "password";
        let hashed_password = hash_password(password).unwrap();
        let id = Uuid::now_v7();
        let now = Local::now().fixed_offset();
        let user = User {
            id,
            user_name: username.to_string(),
            password: hashed_password,
            created_date: Some(now),
            updated_date: Some(now),
            deleted_date: None,
            is_2faenabled: Some(false),
        };
        let user_profile = UserProfile {
            id,
            user_id: id,
            first_name: None,
            last_name: None,
            address: None,
            email: None,
        };
        // create user on db
        sqlx::query(
            r#"
        INSERT INTO public.user (id, user_name, password, created_date, updated_date)
        VALUES ($1, $2, $3, $4, $5)
        "#,
        )
        .bind(user.id)
        .bind(&user.user_name)
        .bind(&user.password)
        .bind(user.created_date)
        .bind(user.updated_date)
        .execute(&mut *tx)
        .await?;
        sqlx::query(
            r#"
        INSERT INTO public.user_profile (id, user_id, first_name, last_name, address, email)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
        )
        .bind(user.id)
        .bind(user.id)
        .bind(&user_profile.first_name)
        .bind(&user_profile.last_name)
        .bind(&user_profile.address)
        .bind(&user_profile.email)
        .execute(&mut *tx)
        .await?;

        // When
        let token = generate_token_from_user(user.clone(), config.clone()).await?;
        add_session(
            &mut redis_conn,
            &user,
            &config,
            token.clone(),
            "".to_string(),
        )?;
        let token_user = get_user_from_token(&mut tx, &mut redis_conn, Some(token)).await?;
        assert!(token_user.is_some());
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClaimsRefresh {
    pub id: String,
    pub user_name: String,
    pub exp: i64,
    pub type_key: String,
}

impl ClaimsRefresh {
    pub fn new(user_id: &str, user_name: &str, config: Config) -> Self {
        let exp = (Local::now() + Duration::minutes(config.jwt_refresh_exp as i64)).timestamp();

        Self {
            id: user_id.to_string(),
            user_name: user_name.to_string(),
            exp,
            type_key: "refresh".to_string(),
        }
    }
}

/// Generate refresh token
pub fn encode_refresh_token(claims: &ClaimsRefresh, jwt_secret: String) -> anyhow::Result<String> {
    let keys = Keys::new(jwt_secret.as_bytes());
    let token = encode(&Header::default(), &claims, &keys.encoding)?;
    Ok(token)
}

/// Extract payload and Validate referesh token
pub fn decode_refresh_token(token: &str, jwt_secret: String) -> anyhow::Result<ClaimsRefresh> {
    let keys = Keys::new(jwt_secret.as_bytes());
    let token_data = decode::<ClaimsRefresh>(token, &keys.decoding, &Validation::default())?;
    Ok(token_data.claims)
}

pub async fn generate_refresh_token_from_user(
    user: User,
    config: Config,
) -> anyhow::Result<String> {
    let claims = ClaimsRefresh::new(
        user.id.to_string().as_str(),
        user.user_name.as_str(),
        config.clone(),
    );
    let token = encode_refresh_token(&claims, config.jwt_secret)?;
    Ok(token)
}

pub async fn get_user_from_refresh_token(
    tx: &mut Transaction<'_, Postgres>,
    refresh_token: Option<String>,
    config: Config,
) -> anyhow::Result<Option<User>> {
    if refresh_token.is_none() {
        return Ok(None);
    }
    let claims = decode_refresh_token(refresh_token.unwrap().as_str(), config.jwt_secret)?;
    let user_id = Uuid::parse_str(&claims.id)?;
    let (user, _) = get_user_by_id(tx, &user_id).await?;
    Ok(user)
}

#[cfg(test)]
mod test_generate_refresh_token {
    use chrono::Local;
    use sqlx::PgPool;
    use uuid::Uuid;

    use crate::{
        core::security::{
            generate_refresh_token_from_user, get_user_from_refresh_token, hash_password,
        },
        model::{user::User, user_profile::UserProfile},
        settings::get_config,
    };

    #[sqlx::test]
    async fn test_generate_refresh_token(pool: PgPool) -> anyhow::Result<()> {
        // Given
        let config = get_config();
        let mut tx = pool.begin().await?;
        // Prepare user
        let username = "hello".to_string();
        let password = "password";
        let hashed_password = hash_password(password).unwrap();
        let id = Uuid::now_v7();
        let now = Local::now().fixed_offset();
        let user = User {
            id,
            user_name: username.to_string(),
            password: hashed_password,
            created_date: Some(now),
            updated_date: Some(now),
            deleted_date: None,
            is_2faenabled: Some(false),
        };
        let user_profile = UserProfile {
            id,
            user_id: id,
            first_name: None,
            last_name: None,
            address: None,
            email: None,
        };
        // create user on db
        sqlx::query(
            r#"
        INSERT INTO public.user (id, user_name, password, created_date, updated_date)
        VALUES ($1, $2, $3, $4, $5)
        "#,
        )
        .bind(user.id)
        .bind(&user.user_name)
        .bind(&user.password)
        .bind(user.created_date)
        .bind(user.updated_date)
        .execute(&mut *tx)
        .await?;
        sqlx::query(
            r#"
        INSERT INTO public.user_profile (id, user_id, first_name, last_name, address, email)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
        )
        .bind(user.id)
        .bind(user.id)
        .bind(&user_profile.first_name)
        .bind(&user_profile.last_name)
        .bind(&user_profile.address)
        .bind(&user_profile.email)
        .execute(&mut *tx)
        .await?;

        // When
        let token = generate_refresh_token_from_user(user.clone(), config.clone()).await?;
        let token_user = get_user_from_refresh_token(&mut tx, Some(token), config).await?;
        assert!(token_user.is_some());
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserApiKey {
    pub token: Option<String>,
}

/// ApiKey authorization
#[derive(SecurityScheme)]
#[oai(ty = "bearer", checker = "bearer_checker")]
pub struct BearerAuthorization(pub UserApiKey);

pub async fn bearer_checker(_req: &Request, api_key: Bearer) -> Option<UserApiKey> {
    Some(UserApiKey {
        token: Some(api_key.token),
    })
}
