use std::sync::Arc;

use poem::{http::StatusCode, test::TestClient};
use serde_json::json;
use sqlx::PgPool;

use crate::{
    core::test_utils::generate_test_user,
    factory::{permission::PermissionFactory, permission_attribute::PermissionAttributeFactory},
    init_openapi_route,
    settings::get_config,
    AppState,
};

#[sqlx::test]
async fn user_permission_test(pool: PgPool) -> anyhow::Result<()> {
    // Given
    let mut config = get_config();
    config.prefix = Some("/api".to_string());
    let client = redis::Client::open(config.redis_url.clone()).unwrap();
    let redis_pool = r2d2::Pool::builder().build(client).unwrap();
    let app_state = Arc::new(AppState {
        db: pool,
        redis_conn: redis_pool,
    });
    let mut db = app_state.db.acquire().await?;
    let mut redis_conn = app_state.redis_conn.get()?;
    let test_user = generate_test_user(
        &mut db,
        &mut redis_conn,
        config.clone(),
        "test_user",
        "password",
    )
    .await?;
    let user = test_user.user;
    let mut permission_factory = PermissionFactory::new();
    let permission = permission_factory.generate_one(&app_state.db, ()).await?;
    let mut attribute_factory = PermissionAttributeFactory::new();
    let attribute = attribute_factory.generate_one(&app_state.db, ()).await?;
    let app = init_openapi_route(app_state.clone(), &config);
    let cli = TestClient::new(app);

    // When Create
    let resp = cli
        .post("/api/user-permissions")
        .header("authorization", format!("Bearer {}", test_user.token))
        .body_json(&json!({
            "user_id": user.id.to_string(),
            "permission_id": permission.id.to_string(),
            "attribute_id": attribute.id.to_string(),
        }))
        .send()
        .await;

    // Expect Create
    resp.assert_status(StatusCode::CREATED);

    // When List
    let resp = cli
        .get("/api/user-permissions")
        .query("user_id", &user.id.to_string())
        .header("authorization", format!("Bearer {}", test_user.token))
        .send()
        .await;

    // Expect List
    resp.assert_status_is_ok();
    resp.assert_json(&json!({
        "counts": 1,
        "page": 1,
        "page_count": 1,
        "page_size": 10,
        "results": [
            {
                "user": {
                    "id": user.id.to_string(),
                    "user_name": user.user_name
                },
                "permission": {
                    "id": permission.id.to_string(),
                    "permission_name": permission.permission_name,
                },
                "permission_attribute": {
                    "id": attribute.id.to_string(),
                    "name": attribute.name
                }
            }
        ]
    }))
    .await;

    // When Delete
    let resp = cli
        .delete("/api/user-permissions")
        .header("authorization", format!("Bearer {}", test_user.token))
        .query("user_id", &user.id.to_string())
        .query("permission_id", &permission.id.to_string())
        .query("attribute_id", &attribute.id.to_string())
        .send()
        .await;

    // Expect Delete
    resp.assert_status(StatusCode::NO_CONTENT);

    // When List 2
    let resp = cli
        .get("/api/user-permissions")
        .query("user_id", &user.id.to_string())
        .header("authorization", format!("Bearer {}", test_user.token))
        .send()
        .await;

    // Expect List 2
    resp.assert_status_is_ok();
    resp.assert_json(&json!({
        "counts": 0,
        "page": 1,
        "page_count": 0,
        "page_size": 10,
        "results": []
    }))
    .await;
    Ok(())
}
