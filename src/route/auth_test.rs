use std::sync::Arc;

use poem::{http::StatusCode, test::TestClient};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    core::security::{get_user_from_token, hash_password},
    factory::{user::UserFactory, user_profile::UserProfileFactory},
    init_openapi_route,
    model::{user::User, user_profile::UserProfile},
    settings::get_config,
    AppState,
};

#[sqlx::test]
async fn test_login_then_logout(pool: PgPool) -> anyhow::Result<()> {
    // Given
    let mut config = get_config();
    config.prefix = Some("/api".to_string());
    let client = redis::Client::open(config.redis_url.clone()).unwrap();
    let redis_pool = r2d2::Pool::builder().build(client).unwrap();
    let app_state = Arc::new(AppState {
        db: pool,
        redis_conn: redis_pool,
    });
    let mut user_factory = UserFactory::<Uuid>::new();
    user_factory.modified_one(|data, ext| User {
        id: ext,
        user_name: "test_user".to_string(),
        password: hash_password("password").unwrap(),
        is_active: Some(true),
        is_2faenabled: Some(false),
        created_by: None,
        updated_by: None,
        created_date: data.created_date,
        updated_date: data.updated_date,
        deleted_date: None,
    });
    let user_id = Uuid::now_v7();
    user_factory
        .generate_one(&app_state.db, user_id.clone())
        .await?;
    let mut user_profile_factory = UserProfileFactory::<Uuid>::new();
    user_profile_factory.modified_one(|data, ext| UserProfile {
        id: data.id,
        user_id: ext,
        first_name: data.first_name.clone(),
        last_name: data.last_name.clone(),
        address: data.address.clone(),
        email: data.email.clone(),
    });
    user_profile_factory
        .generate_one(&app_state.db, user_id)
        .await?;
    let app = init_openapi_route(app_state.clone(), &config);
    let cli = TestClient::new(app);

    // When login
    let json_payload = json!({
        "user_name": "test_user",
        "password": "password"
    });
    let resp = cli
        .post("/api/auth/login")
        .body_json(&json_payload)
        .send()
        .await;

    // Expect login
    resp.assert_status_is_ok();
    let json = resp.json().await;
    let token = json.value().object().get_opt("token");
    assert!(token.is_some());
    let token: String = token.unwrap().deserialize();
    let mut tx = app_state.db.begin().await?;
    let mut redis_conn = app_state.redis_conn.get().unwrap();
    let user_in_token = get_user_from_token(&mut tx, &mut redis_conn, Some(token.clone())).await?;
    assert!(user_in_token.is_some());
    assert_eq!(user_in_token.unwrap().id, user_id);
    let res: Option<String> = redis::cmd("GET").arg(&token).query(&mut redis_conn)?;
    assert!(res.is_some());

    // When logout
    let resp = cli
        .post("/api/auth/logout")
        .header("authorization", format!("Bearer {}", token))
        .send()
        .await;

    // Expect logout
    resp.assert_status(StatusCode::NO_CONTENT);
    let res: Option<String> = redis::cmd("GET").arg(&token).query(&mut redis_conn)?;
    assert!(res.is_none());

    // When second logout
    let resp = cli
        .post("/api/auth/logout")
        .header("authorization", format!("Bearer {}", token))
        .send()
        .await;

    // Expect second logout
    resp.assert_status(StatusCode::UNAUTHORIZED);
    Ok(())
}

#[sqlx::test]
async fn test_login_then_refresh(pool: PgPool) -> anyhow::Result<()> {
    // Given
    let mut config = get_config();
    config.prefix = Some("/api".to_string());
    let client = redis::Client::open(config.redis_url.clone()).unwrap();
    let redis_pool = r2d2::Pool::builder().build(client).unwrap();
    let app_state = Arc::new(AppState {
        db: pool,
        redis_conn: redis_pool,
    });
    let mut user_factory = UserFactory::<Uuid>::new();
    user_factory.modified_one(|data, ext| User {
        id: ext,
        user_name: "test_user".to_string(),
        password: hash_password("password").unwrap(),
        is_active: Some(true),
        is_2faenabled: Some(false),
        created_by: None,
        updated_by: None,
        created_date: data.created_date,
        updated_date: data.updated_date,
        deleted_date: None,
    });
    let user_id = Uuid::now_v7();
    user_factory
        .generate_one(&app_state.db, user_id.clone())
        .await?;
    let mut user_profile_factory = UserProfileFactory::<Uuid>::new();
    user_profile_factory.modified_one(|data, ext| UserProfile {
        id: data.id,
        user_id: ext,
        first_name: data.first_name.clone(),
        last_name: data.last_name.clone(),
        address: data.address.clone(),
        email: data.email.clone(),
    });
    user_profile_factory
        .generate_one(&app_state.db, user_id)
        .await?;
    let app = init_openapi_route(app_state.clone(), &config);
    let cli = TestClient::new(app);

    // When login
    let json_payload = json!({
        "user_name": "test_user",
        "password": "password"
    });
    let resp = cli
        .post("/api/auth/login")
        .body_json(&json_payload)
        .send()
        .await;

    // Expect login
    resp.assert_status_is_ok();
    let json = resp.json().await;
    let refresh_token = json.value().object().get_opt("refresh_token");
    assert!(refresh_token.is_some());
    let refresh_token: String = refresh_token.unwrap().deserialize();

    // When Refresh
    let json_payload = json!({
        "refresh_token": refresh_token,
    });
    let resp = cli
        .post("/api/auth/refresh-token")
        .body_json(&json_payload)
        .send()
        .await;

    // Expect Refresh
    resp.assert_status_is_ok();
    let json = resp.json().await;
    let token = json.value().object().get_opt("token");
    assert!(token.is_some());
    let token: String = token.unwrap().deserialize();
    let mut tx = app_state.db.begin().await?;
    let mut redis_conn = app_state.redis_conn.get().unwrap();
    let user_in_token = get_user_from_token(&mut tx, &mut redis_conn, Some(token.clone())).await?;
    assert!(user_in_token.is_some());
    assert_eq!(user_in_token.unwrap().id, user_id);
    let res: Option<String> = redis::cmd("GET").arg(&token).query(&mut redis_conn)?;
    assert!(res.is_some());

    // When logout
    let resp = cli
        .post("/api/auth/logout")
        .header("authorization", format!("Bearer {}", token))
        .send()
        .await;

    // Expect logout
    resp.assert_status(StatusCode::NO_CONTENT);
    Ok(())
}
