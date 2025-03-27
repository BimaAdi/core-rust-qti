use super::security::{generate_refresh_token_from_user, generate_token_from_user};
use crate::core::security::hash_password;
use crate::core::session::add_session;
use crate::model::user::User;
use crate::model::user_profile::UserProfile;
use crate::settings::Config;
use chrono::Local;
use fake::{Fake, Faker};
use redis::ConnectionLike;
use sqlx::pool::PoolConnection;
use sqlx::Postgres;
use uuid::Uuid;

pub fn generate_random<T: fake::Dummy<fake::Faker>>() -> T {
    Faker.fake()
}

pub struct TestUser {
    pub user: User,
    pub user_profile: UserProfile,
    pub token: String,
    pub refresh_token: String,
}

pub async fn generate_test_user<C: ConnectionLike>(
    db: &mut PoolConnection<Postgres>,
    redis_conn: &mut C,
    config: Config,
    username: &str,
    password: &str,
) -> anyhow::Result<TestUser> {
    // Prepare user
    let hashed_password = hash_password(password).unwrap();
    let id = Uuid::now_v7();
    let now = Local::now().fixed_offset();
    let user = User {
        id,
        user_name: username.to_string(),
        password: hashed_password,
        is_active: Some(true),
        is_2faenabled: Some(false),
        created_by: None,
        updated_by: None,
        created_date: Some(now),
        updated_date: Some(now),
        deleted_date: None,
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
        INSERT INTO public.user (id, user_name, password, is_active, is_2faenabled, created_date, updated_date)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
    )
    .bind(user.id)
    .bind(&user.user_name)
    .bind(&user.password)
    .bind(user.is_active)
    .bind(user.is_2faenabled)
    .bind(user.created_date)
    .bind(user.updated_date)
    .execute(&mut **db)
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
    .execute(&mut **db)
    .await?;

    // Generate token
    let token = generate_token_from_user(user.clone(), config.clone()).await?;
    let refresh_token = generate_refresh_token_from_user(user.clone(), config.clone()).await?;
    add_session(
        redis_conn,
        &user,
        &config,
        token.clone(),
        refresh_token.clone(),
    )?;

    Ok(TestUser {
        user,
        user_profile: UserProfile {
            id,
            user_id: id,
            first_name: None,
            last_name: None,
            address: None,
            email: None,
        },
        token,
        refresh_token,
    })
}

#[cfg(test)]
mod tests {
    use sqlx::{Acquire, PgPool};
    use uuid::Uuid;

    use crate::{
        core::{
            security::get_user_from_token, session::get_session, test_utils::generate_test_user,
        },
        settings::get_config,
    };

    #[sqlx::test]
    async fn test_generate_test_user(pool: PgPool) -> anyhow::Result<()> {
        // Given
        let config = get_config();
        let client = redis::Client::open(config.redis_url.clone()).unwrap();
        let mut redis_conn = client.get_connection().unwrap();

        // When
        let mut db = pool.acquire().await?;
        let res = generate_test_user(
            &mut db,
            &mut redis_conn,
            config.clone(),
            "testuser",
            "testpassword",
        )
        .await?;

        // Expect
        // is user exists on db
        let user: Option<(Uuid, String)> =
            sqlx::query_as("SELECT id, user_name FROM public.user WHERE id = $1")
                .bind(&res.user.id)
                .fetch_optional(&mut *db)
                .await?;
        assert!(user.is_some());
        let user_profile: Option<(Uuid,)> =
            sqlx::query_as("SELECT id FROM public.user_profile WHERE user_id = $1")
                .bind(&res.user.id)
                .fetch_optional(&mut *db)
                .await?;
        assert!(user_profile.is_some());

        // is jwt token valid
        let mut tx = db.begin().await?;
        let user_token =
            get_user_from_token(&mut tx, &mut redis_conn, Some(res.token.clone())).await?;
        assert!(user_token.is_some());
        assert_eq!(user_token.unwrap().user_name, "testuser".to_string());

        // is user exists on redis
        let session = get_session(&mut redis_conn, res.token)?;
        assert!(session.is_some());
        Ok(())
    }
}
