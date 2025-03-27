use std::sync::Arc;

use poem::{http::StatusCode, test::TestClient};
use serde_json::{json, Value::Null};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    core::{
        security::verify_hash_password, test_utils::generate_test_user, utils::datetime_to_string,
    },
    factory::{group::GroupFactory, role::RoleFactory},
    init_openapi_route,
    model::{
        user::{User, TABLE_NAME},
        user_group_roles::{UserGroupRoles, TABLE_NAME as USER_GROUP_ROLES_TABLE_NAME},
        user_profile::{UserProfile, TABLE_NAME as USER_PROFILE_TABLE_NAME},
    },
    settings::get_config,
    AppState,
};

#[sqlx::test]
async fn test_user_detail_api(pool: PgPool) -> anyhow::Result<()> {
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
    // let mut role_factory = RoleFactory::new();
    // let role = role_factory.generate_one(&app_state.db, ()).await?;
    // let mut group_factory = GroupFactory::new();
    // let group = group_factory.generate_one(&app_state.db, ()).await?;
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
        .get("/api/user/detail")
        .header("authorization", format!("Bearer {}", test_user.token))
        .query("id", &test_user.user.id.to_string())
        .send()
        .await;

    // Expect
    resp.assert_status_is_ok();
    let user = test_user.user;
    let user_profile = test_user.user_profile;
    resp.assert_json(&json!({
        "id": user.id.to_string(),
        "user_name": user.user_name,
        "is_active": user.is_active,
        "is_2faenabled": user.is_2faenabled,
        "created_by": Null,
        "updated_by": Null,
        "created_date": datetime_to_string(user.created_date.unwrap()),
        "updated_date": datetime_to_string(user.updated_date.unwrap()),
        "user_profile": {
            "address": user_profile.address,
            "email": user_profile.email,
            "first_name": user_profile.first_name,
            "last_name": user_profile.last_name
        },
        "group_roles": []
    }))
    .await;
    Ok(())
}

#[sqlx::test]
async fn test_create_user_api(pool: PgPool) -> anyhow::Result<()> {
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
    let mut role_factory = RoleFactory::new();
    let role = role_factory.generate_one(&app_state.db, ()).await?;
    let mut group_factory = GroupFactory::new();
    let group = group_factory.generate_one(&app_state.db, ()).await?;
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
        .post("/api/user")
        .header("authorization", format!("Bearer {}", test_user.token))
        .body_json(&json!({
            "first_name": "first",
            "last_name": "last",
            "email": "email@local.com",
            "is_active": true,
            "password": "password",
            "user_name": "user_name",
            "address": Null,
            "group_roles": [
                {
                    "group_id": group.id.to_string(),
                    "role_id": role.id.to_string(),
                }
            ]
        }))
        .send()
        .await;

    // Expect
    resp.assert_status(StatusCode::CREATED);
    let json = resp.json().await;
    let new_user_id = json.value().object().get_opt("id");
    assert!(new_user_id.is_some());
    let new_user_id: Uuid = new_user_id.unwrap().deserialize();
    // user created success
    let new_user: Option<User> =
        sqlx::query_as(format!(r#"SELECT * FROM {} WHERE id = $1"#, TABLE_NAME).as_str())
            .bind(new_user_id)
            .fetch_optional(&mut *db)
            .await?;
    assert!(new_user.is_some());
    let new_user = new_user.unwrap();
    assert_eq!(new_user.user_name, "user_name".to_string());
    assert_eq!(new_user.is_active, Some(true));
    assert!(verify_hash_password("password", &new_user.password).unwrap());
    // user profile
    let new_user_profile: Option<UserProfile> = sqlx::query_as(
        format!(
            r#"SELECT * FROM {} WHERE user_id = $1"#,
            USER_PROFILE_TABLE_NAME
        )
        .as_str(),
    )
    .bind(new_user_id)
    .fetch_optional(&mut *db)
    .await?;
    assert!(new_user_profile.is_some());
    let new_user_profile = new_user_profile.unwrap();
    assert_eq!(new_user_profile.first_name, Some("first".to_string()));
    assert_eq!(new_user_profile.last_name, Some("last".to_string()));
    assert_eq!(new_user_profile.email, Some("email@local.com".to_string()));
    // user_group_roles
    let user_group_roles: Vec<UserGroupRoles> = sqlx::query_as(
        format!(
            r#"SELECT * FROM {} WHERE user_id = $1"#,
            USER_GROUP_ROLES_TABLE_NAME
        )
        .as_str(),
    )
    .bind(new_user_id)
    .fetch_all(&mut *db)
    .await?;
    assert_eq!(user_group_roles.len(), 1);
    assert_eq!(user_group_roles[0].role_id, Some(role.id));
    assert_eq!(user_group_roles[0].group_id, Some(group.id));
    Ok(())
}
