use std::sync::Arc;

use poem::{http::StatusCode, test::TestClient};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    core::test_utils::generate_test_user,
    factory::permission_attribute::PermissionAttributeFactory,
    init_openapi_route,
    model::{
        permission::{Permission, TABLE_NAME},
        permission_attribute_list::{
            PermissionAttributeList, TABLE_NAME as PERMISSION_ATTRIBUTE_LIST_TABLE_NAME,
        },
    },
    settings::get_config,
    AppState,
};

#[sqlx::test]
async fn test_create_permission_api(pool: PgPool) -> anyhow::Result<()> {
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
    let mut attribute_factory = PermissionAttributeFactory::new();
    let attributes = attribute_factory
        .generate_many(&app_state.db, 2, ())
        .await?;
    let mut redis_conn = app_state.redis_conn.get()?;
    let test_user = generate_test_user(
        &mut db,
        &mut redis_conn,
        config.clone(),
        "test_user",
        "password",
    )
    .await?;
    let app = init_openapi_route(app_state.clone(), &config);
    let cli = TestClient::new(app);

    // When
    let resp = cli
        .post("/api/permissions")
        .header("authorization", format!("Bearer {}", test_user.token))
        .body_json(&json!({
            "permission_name": "new_permission",
            "description": "description",
            "is_user": true,
            "is_role": true,
            "is_group": true,
            "permission_attribute_ids": attributes.iter().map(|x| x.id.to_string()).collect::<Vec<String>>(),
        }))
        .send()
        .await;

    // Expect
    resp.assert_status(StatusCode::CREATED);
    let json = resp.json().await;
    let new_permission_id = json.value().object().get_opt("id");
    assert!(new_permission_id.is_some());
    let new_permission_id: Uuid = new_permission_id.unwrap().deserialize();
    let new_permission: Option<Permission> =
        sqlx::query_as(format!("SELECT * FROM {} WHERE id = $1", TABLE_NAME).as_str())
            .bind(&new_permission_id)
            .fetch_optional(&mut *db)
            .await?;
    assert!(new_permission.is_some());
    let new_permission = new_permission.unwrap();
    assert_eq!(new_permission.permission_name, "new_permission".to_string());
    assert_eq!(new_permission.description, Some("description".to_string()));
    assert!(new_permission.is_user.unwrap_or(false));
    assert!(new_permission.is_role.unwrap_or(false));
    assert!(new_permission.is_group.unwrap_or(false));
    let permission_atribute_list: Vec<PermissionAttributeList> = sqlx::query_as(
        format!(
            "SELECT * FROM {} WHERE permission_id = $1",
            PERMISSION_ATTRIBUTE_LIST_TABLE_NAME
        )
        .as_str(),
    )
    .bind(&new_permission.id)
    .fetch_all(&mut *db)
    .await?;
    assert_eq!(permission_atribute_list.len(), 2);
    for item in attributes {
        let permission_atribute_list: Option<PermissionAttributeList> = sqlx::query_as(
            format!(
                "SELECT * FROM {} WHERE permission_id = $1 AND attribute_id = $2",
                PERMISSION_ATTRIBUTE_LIST_TABLE_NAME
            )
            .as_str(),
        )
        .bind(&new_permission.id)
        .bind(&item.id)
        .fetch_optional(&mut *db)
        .await?;
        assert!(permission_atribute_list.is_some());
    }
    Ok(())
}
