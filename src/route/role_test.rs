use std::{cmp::Ordering, sync::Arc};

use chrono::{DateTime, FixedOffset};
use poem::{http::StatusCode, test::TestClient};
use serde_json::{json, Value, Value::Null};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    core::{
        test_utils::{generate_random, generate_test_user},
        utils::datetime_to_string_opt,
    },
    factory::role::RoleFactory,
    init_openapi_route,
    model::{
        role::{Role, TABLE_NAME},
        user::User,
    },
    repository::user::get_user_by_id,
    schema::role::{DetailRolePagination, RoleAllResponse, RoleDetailUser},
    settings::get_config,
    AppState,
};

#[sqlx::test]
async fn test_paginate_role_api(pool: PgPool) -> anyhow::Result<()> {
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
    let mut role_factory = RoleFactory::new();
    role_factory.modified_many(|data, _, _| Role {
        id: data.id,
        role_name: data.role_name.clone(),
        description: data.description.clone(),
        is_active: data.is_active,
        created_by: data.created_by,
        updated_by: data.updated_by,
        created_date: data.created_date,
        updated_date: Some(generate_random::<DateTime<FixedOffset>>()),
        deleted_date: None,
    });
    let mut roles = role_factory.generate_many(&app_state.db, 10, ()).await?;
    let app = init_openapi_route(app_state.clone(), &config);
    let cli = TestClient::new(app);

    // When
    let resp = cli
        .get("/api/role")
        .header("authorization", format!("Bearer {}", test_user.token))
        .send()
        .await;

    // Expect
    resp.assert_status_is_ok();
    roles.sort_by(|a, b| {
        if a.updated_date > b.updated_date {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    });
    let mut tx = app_state.db.begin().await?;
    let mut results: Vec<DetailRolePagination> = vec![];
    for item in roles {
        let mut created_by: Option<User> = None;
        if let Some(created_by_id) = item.created_by {
            (created_by, _) = get_user_by_id(&mut tx, &created_by_id, None).await?;
        }
        let mut updated_by: Option<User> = None;
        if let Some(updated_by_id) = item.updated_by {
            (updated_by, _) = get_user_by_id(&mut tx, &updated_by_id, None).await?;
        }
        results.push(DetailRolePagination {
            id: item.id.to_string(),
            role_name: item.role_name,
            description: item.description,
            is_active: item.is_active,
            created_by: match created_by {
                Some(val) => Some(RoleDetailUser {
                    id: val.id.to_string(),
                    user_name: val.user_name,
                }),
                None => None,
            },
            updated_by: match updated_by {
                Some(val) => Some(RoleDetailUser {
                    id: val.id.to_string(),
                    user_name: val.user_name,
                }),
                None => None,
            },
            created_date: datetime_to_string_opt(item.created_date),
            updated_date: datetime_to_string_opt(item.updated_date),
        });
    }
    resp.assert_json(&json!({
        "counts": 10,
        "page": 1,
        "page_count": 1,
        "page_size": 10,
        "results": results
    }))
    .await;
    Ok(())
}

#[sqlx::test]
async fn test_get_all_role_api(pool: PgPool) -> anyhow::Result<()> {
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
    let mut role_factory = RoleFactory::new();
    role_factory.modified_many(|data, _, _| Role {
        id: data.id,
        role_name: data.role_name.clone(),
        description: data.description.clone(),
        is_active: data.is_active,
        created_by: data.created_by,
        updated_by: data.updated_by,
        created_date: data.created_date,
        updated_date: Some(generate_random::<DateTime<FixedOffset>>()),
        deleted_date: None,
    });
    let mut roles = role_factory.generate_many(&app_state.db, 10, ()).await?;
    let app = init_openapi_route(app_state.clone(), &config);
    let cli = TestClient::new(app);

    // When
    let resp = cli
        .get("/api/role/all")
        .header("authorization", format!("Bearer {}", test_user.token))
        .send()
        .await;

    // Expect
    resp.assert_status_is_ok();
    roles.sort_by(|a, b| {
        if a.updated_date > b.updated_date {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    });
    let mut tx = app_state.db.begin().await?;
    let mut results: Vec<RoleAllResponse> = vec![];
    for item in roles {
        let mut created_by: Option<User> = None;
        if let Some(created_by_id) = item.created_by {
            (created_by, _) = get_user_by_id(&mut tx, &created_by_id, None).await?;
        }
        let mut updated_by: Option<User> = None;
        if let Some(updated_by_id) = item.updated_by {
            (updated_by, _) = get_user_by_id(&mut tx, &updated_by_id, None).await?;
        }
        results.push(RoleAllResponse {
            id: item.id.to_string(),
            role_name: item.role_name,
            description: item.description,
            is_active: item.is_active,
            created_by: match created_by {
                Some(val) => Some(RoleDetailUser {
                    id: val.id.to_string(),
                    user_name: val.user_name,
                }),
                None => None,
            },
            updated_by: match updated_by {
                Some(val) => Some(RoleDetailUser {
                    id: val.id.to_string(),
                    user_name: val.user_name,
                }),
                None => None,
            },
            created_date: datetime_to_string_opt(item.created_date),
            updated_date: datetime_to_string_opt(item.updated_date),
        });
    }
    resp.assert_json(results).await;
    Ok(())
}

#[sqlx::test]
async fn test_dropdown_role_api(pool: PgPool) -> anyhow::Result<()> {
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
    let mut role_factory = RoleFactory::new();
    role_factory.modified_many(|data, _, _| Role {
        id: data.id,
        role_name: data.role_name.clone(),
        description: data.description.clone(),
        is_active: data.is_active,
        created_by: data.created_by,
        updated_by: data.updated_by,
        created_date: data.created_date,
        updated_date: Some(generate_random::<DateTime<FixedOffset>>()),
        deleted_date: None,
    });
    let mut roles = role_factory.generate_many(&app_state.db, 10, ()).await?;
    let app = init_openapi_route(app_state.clone(), &config);
    let cli = TestClient::new(app);

    // When
    let resp = cli
        .get("/api/role/dropdown")
        .header("authorization", format!("Bearer {}", test_user.token))
        .send()
        .await;

    // Expect
    resp.assert_status_is_ok();
    roles.sort_by(|a, b| {
        if a.updated_date > b.updated_date {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    });
    let mut results: Vec<Value> = vec![];
    for item in roles {
        results.push(json!( {
            "id": item.id.to_string(),
            "role_name": item.role_name,
        }));
    }
    resp.assert_json(results).await;
    Ok(())
}

#[sqlx::test]
async fn test_get_detail_role_api(pool: PgPool) -> anyhow::Result<()> {
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
    let mut role_factory = RoleFactory::new();
    role_factory.modified_one(|data, _| Role {
        id: data.id,
        role_name: data.role_name.clone(),
        description: data.description.clone(),
        is_active: data.is_active,
        created_by: data.created_by,
        updated_by: data.updated_by,
        created_date: data.created_date,
        updated_date: Some(generate_random::<DateTime<FixedOffset>>()),
        deleted_date: None,
    });
    let role = role_factory.generate_one(&app_state.db, ()).await?;
    let app = init_openapi_route(app_state.clone(), &config);
    let cli = TestClient::new(app);

    // When 1
    let resp = cli
        .get("/api/role/detail")
        .query("id", &role.id.to_string())
        .header("authorization", format!("Bearer {}", test_user.token))
        .send()
        .await;

    // Expect 1
    resp.assert_status_is_ok();
    resp.assert_json(&json!({
        "id": role.id.to_string(),
        "role_name": role.role_name,
        "description": role.description,
        "is_active": role.is_active,
        "created_date": datetime_to_string_opt(role.created_date),
        "updated_date": datetime_to_string_opt(role.updated_date),
        "created_by": Null,
        "updated_by": Null,
    }))
    .await;

    // When 2
    let resp = cli
        .get("/api/role/detail")
        .query("id", &"aaaa-bbbb-cccc")
        .header("authorization", format!("Bearer {}", test_user.token))
        .send()
        .await;

    // Expect 2
    resp.assert_status(StatusCode::NOT_FOUND);
    Ok(())
}

#[sqlx::test]
async fn test_create_role_api(pool: PgPool) -> anyhow::Result<()> {
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
    let app = init_openapi_route(app_state.clone(), &config);
    let cli = TestClient::new(app);

    // When
    let resp = cli
        .post("/api/role")
        .header("authorization", format!("Bearer {}", test_user.token))
        .body_json(&json!({
            "role_name": "new_role",
            "description": "role description",
            "is_active": true
        }))
        .send()
        .await;

    // Expect
    resp.assert_status(StatusCode::CREATED);
    let json = resp.json().await;
    let new_role_id = json.value().object().get_opt("id");
    assert!(new_role_id.is_some());
    let new_role_id: Uuid = new_role_id.unwrap().deserialize();
    let new_role: Option<(String, Option<String>, Option<bool>)> = sqlx::query_as(
        format!(
            r#"
    SELECT role_name, description, is_active
    FROM {}
    WHERE id = $1"#,
            TABLE_NAME
        )
        .as_str(),
    )
    .bind(new_role_id)
    .fetch_optional(&mut *db)
    .await?;
    assert!(new_role.is_some());
    let new_role = new_role.unwrap();
    assert_eq!(new_role.0, "new_role".to_string());
    assert_eq!(new_role.1, Some("role description".to_string()));
    assert_eq!(new_role.2, Some(true));
    Ok(())
}

#[sqlx::test]
async fn test_update_role_api(pool: PgPool) -> anyhow::Result<()> {
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
    let mut role_factory = RoleFactory::new();
    role_factory.modified_one(|data, _| Role {
        id: data.id,
        role_name: data.role_name.clone(),
        description: data.description.clone(),
        is_active: data.is_active,
        created_by: data.created_by,
        updated_by: data.updated_by,
        created_date: data.created_date,
        updated_date: Some(generate_random::<DateTime<FixedOffset>>()),
        deleted_date: None,
    });
    let role = role_factory.generate_one(&app_state.db, ()).await?;
    let app = init_openapi_route(app_state.clone(), &config);
    let cli = TestClient::new(app);

    // When 1
    let resp = cli
        .put("/api/role")
        .query("id", &role.id.to_string())
        .header("authorization", format!("Bearer {}", test_user.token))
        .body_json(&json!({
            "role_name": "update role",
            "description": "role description",
            "is_active": true
        }))
        .send()
        .await;

    // Expect 1
    resp.assert_status_is_ok();
    let updated_role: Option<(String, Option<String>, Option<bool>)> = sqlx::query_as(
        format!(
            r#"
    SELECT role_name, description, is_active
    FROM {}
    WHERE id = $1"#,
            TABLE_NAME
        )
        .as_str(),
    )
    .bind(role.id)
    .fetch_optional(&mut *db)
    .await?;
    assert!(updated_role.is_some());
    let updated_role = updated_role.unwrap();
    assert_eq!(updated_role.0, "update role".to_string());
    assert_eq!(updated_role.1, Some("role description".to_string()));
    assert_eq!(updated_role.2, Some(true));

    // When 2
    let resp = cli
        .put("/api/role")
        .query("id", &"aaaa-bbbb-cccc")
        .header("authorization", format!("Bearer {}", test_user.token))
        .body_json(&json!({
            "role_name": "update role",
            "description": "role description",
            "is_active": true
        }))
        .send()
        .await;

    // Expect 2
    resp.assert_status(StatusCode::NOT_FOUND);
    Ok(())
}

#[sqlx::test]
async fn test_delete_role_api(pool: PgPool) -> anyhow::Result<()> {
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
    let mut role_factory = RoleFactory::new();
    role_factory.modified_one(|data, _| Role {
        id: data.id,
        role_name: data.role_name.clone(),
        description: data.description.clone(),
        is_active: data.is_active,
        created_by: data.created_by,
        updated_by: data.updated_by,
        created_date: data.created_date,
        updated_date: Some(generate_random::<DateTime<FixedOffset>>()),
        deleted_date: None,
    });
    let role = role_factory.generate_one(&app_state.db, ()).await?;
    let app = init_openapi_route(app_state.clone(), &config);
    let cli = TestClient::new(app);

    // When 1
    let resp = cli
        .delete("/api/role")
        .query("id", &role.id.to_string())
        .header("authorization", format!("Bearer {}", test_user.token))
        .send()
        .await;

    // Expect 1
    resp.assert_status(StatusCode::NO_CONTENT);
    let deleted_role: Option<(Option<DateTime<FixedOffset>>,)> = sqlx::query_as(
        format!(
            r#"
    SELECT deleted_date
    FROM {}
    WHERE id = $1"#,
            TABLE_NAME
        )
        .as_str(),
    )
    .bind(role.id)
    .fetch_optional(&mut *db)
    .await?;
    assert!(deleted_role.is_some());
    let deleted_role = deleted_role.unwrap();
    assert!(deleted_role.0.is_some());

    // When 2
    let resp = cli
        .delete("/api/role")
        .query("id", &role.id.to_string())
        .header("authorization", format!("Bearer {}", test_user.token))
        .send()
        .await;

    // Expect 2
    resp.assert_status(StatusCode::NOT_FOUND);
    Ok(())
}
