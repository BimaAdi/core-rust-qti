use std::{cmp::Ordering, sync::Arc};

use poem::{http::StatusCode, test::TestClient};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    core::test_utils::generate_test_user,
    factory::permission_attribute::PermissionAttributeFactory,
    init_openapi_route,
    model::permission_attribute::{PermissionAttribute, TABLE_NAME},
    schema::permission_attribute::DetailPermissionAttribute,
    settings::get_config,
    AppState,
};

#[sqlx::test]
async fn test_paginate_permission_attribute_api(pool: PgPool) -> anyhow::Result<()> {
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
    let mut permission_attribute_factory = PermissionAttributeFactory::new();
    let mut permission_attributes = permission_attribute_factory
        .generate_many(&app_state.db, 5, ())
        .await?;
    let app = init_openapi_route(app_state.clone(), &config);
    let cli = TestClient::new(app);

    // When
    let resp = cli
        .get("/api/permission-attribute")
        .header("authorization", format!("Bearer {}", test_user.token))
        .send()
        .await;

    // Expect
    resp.assert_status_is_ok();
    permission_attributes.sort_by(|a, b| {
        if a.updated_date > b.updated_date {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    });
    resp.assert_json(&json!({
        "counts":5,
        "page":1,
        "page_count":1,
        "page_size":10,
        "results": permission_attributes.iter()
        .map(|x| DetailPermissionAttribute {
            id: x.id.to_string(),
            name: x.name.clone(),
            description: x.description.clone(),
        })
        .collect::<Vec<DetailPermissionAttribute>>(),
    }))
    .await;
    Ok(())
}

#[sqlx::test]
async fn test_dropdown_permission_attribute_api(pool: PgPool) -> anyhow::Result<()> {
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
    let mut permission_attribute_factory = PermissionAttributeFactory::new();
    let mut permission_attributes = permission_attribute_factory
        .generate_many(&app_state.db, 5, ())
        .await?;
    let app = init_openapi_route(app_state.clone(), &config);
    let cli = TestClient::new(app);

    // When
    let resp = cli
        .get("/api/permission-attribute/dropdown")
        .header("authorization", format!("Bearer {}", test_user.token))
        .send()
        .await;

    // Expect
    resp.assert_status_is_ok();
    permission_attributes.sort_by(|a, b| {
        if a.updated_date > b.updated_date {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    });
    resp.assert_json(
        &permission_attributes
            .iter()
            .map(|x| DetailPermissionAttribute {
                id: x.id.to_string(),
                name: x.name.clone(),
                description: x.description.clone(),
            })
            .collect::<Vec<DetailPermissionAttribute>>(),
    )
    .await;
    Ok(())
}

#[sqlx::test]
async fn test_detail_permission_attribute_api(pool: PgPool) -> anyhow::Result<()> {
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
    let mut permission_attribute_factory = PermissionAttributeFactory::new();
    let permission_attribute = permission_attribute_factory
        .generate_one(&app_state.db, ())
        .await?;
    let app = init_openapi_route(app_state.clone(), &config);
    let cli = TestClient::new(app);

    // When
    let resp = cli
        .get("/api/permission-attribute/detail")
        .query("id", &permission_attribute.id.to_string())
        .header("authorization", format!("Bearer {}", test_user.token))
        .send()
        .await;

    // Expect
    resp.assert_status_is_ok();
    let json_response = DetailPermissionAttribute {
        id: permission_attribute.id.to_string(),
        name: permission_attribute.name,
        description: permission_attribute.description,
    };
    resp.assert_json(&json!(&json_response)).await;
    Ok(())
}

#[sqlx::test]
async fn test_create_permission_attribute_api(pool: PgPool) -> anyhow::Result<()> {
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
    let mut permission_attribute_factory = PermissionAttributeFactory::new();
    let _ = permission_attribute_factory
        .generate_one(&app_state.db, ())
        .await?;
    let app = init_openapi_route(app_state.clone(), &config);
    let cli = TestClient::new(app);

    // When
    let resp = cli
        .post("/api/permission-attribute")
        .header("authorization", format!("Bearer {}", test_user.token))
        .body_json(&json!({
            "name": "attribute",
            "description": "some description"
        }))
        .send()
        .await;

    // Expect
    resp.assert_status(StatusCode::CREATED);
    let json = resp.json().await;
    let id = json.value().object().get_opt("id");
    assert!(id.is_some());
    let id: Uuid = id.unwrap().deserialize();
    let new_permission_attribute: Option<PermissionAttribute> =
        sqlx::query_as(format!("SELECT * FROM {} WHERE id = $1", TABLE_NAME).as_str())
            .bind(&id)
            .fetch_optional(&mut *db)
            .await?;
    assert!(new_permission_attribute.is_some());
    let new_permission_attribute = new_permission_attribute.unwrap();
    assert_eq!(new_permission_attribute.name, "attribute");
    assert_eq!(
        new_permission_attribute.description,
        Some("some description".to_string())
    );
    Ok(())
}

#[sqlx::test]
async fn test_update_permission_attribute_api(pool: PgPool) -> anyhow::Result<()> {
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
    let mut permission_attribute_factory = PermissionAttributeFactory::new();
    let permission_attribute = permission_attribute_factory
        .generate_one(&app_state.db, ())
        .await?;
    let app = init_openapi_route(app_state.clone(), &config);
    let cli = TestClient::new(app);

    // When
    let resp = cli
        .put("/api/permission-attribute")
        .query("id", &permission_attribute.id.to_string())
        .header("authorization", format!("Bearer {}", test_user.token))
        .body_json(&json!({
            "name": "attribute",
            "description": "some description"
        }))
        .send()
        .await;

    // Expect
    resp.assert_status_is_ok();
    let updated_permission_attribute: Option<PermissionAttribute> =
        sqlx::query_as(format!("SELECT * FROM {} WHERE id = $1", TABLE_NAME).as_str())
            .bind(&permission_attribute.id)
            .fetch_optional(&mut *db)
            .await?;
    assert!(updated_permission_attribute.is_some());
    let new_permission_attribute = updated_permission_attribute.unwrap();
    assert_eq!(new_permission_attribute.name, "attribute");
    assert_eq!(
        new_permission_attribute.description,
        Some("some description".to_string())
    );
    Ok(())
}

#[sqlx::test]
async fn test_delete_permission_attribute_api(pool: PgPool) -> anyhow::Result<()> {
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
    let mut permission_attribute_factory = PermissionAttributeFactory::new();
    let permission_attribute = permission_attribute_factory
        .generate_one(&app_state.db, ())
        .await?;
    let app = init_openapi_route(app_state.clone(), &config);
    let cli = TestClient::new(app);

    // When
    let resp = cli
        .delete("/api/permission-attribute")
        .query("id", &permission_attribute.id.to_string())
        .header("authorization", format!("Bearer {}", test_user.token))
        .send()
        .await;

    // Expect
    resp.assert_status(StatusCode::NO_CONTENT);
    let deleted_permission_attribute: Option<PermissionAttribute> =
        sqlx::query_as(format!("SELECT * FROM {} WHERE id = $1", TABLE_NAME).as_str())
            .bind(&permission_attribute.id)
            .fetch_optional(&mut *db)
            .await?;
    assert!(deleted_permission_attribute.is_none());
    Ok(())
}
