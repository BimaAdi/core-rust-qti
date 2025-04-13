use std::sync::Arc;

use poem::{
    middleware::{AddData, AddDataEndpoint, Cors, CorsEndpoint},
    EndpointExt, Route,
};
use poem_openapi::OpenApiService;
use r2d2::Pool as r2d2Pool;
use redis::Client;
use route::{
    auth::ApiAuth, group::ApiGroup, group_permission::ApiGroupPermission,
    permission::ApiPermission, permission_attribute::ApiPermissionAttribute, role::ApiRole,
    role_permission::ApiRolePermission, user::ApiUser, user_permission::ApiUserPermission,
};
use settings::Config;
use sqlx::{Pool, Postgres};

pub mod cli;
pub mod core;
pub mod factory;
pub mod model;
pub mod repository;
pub mod route;
pub mod schema;
pub mod settings;

pub struct AppState {
    pub db: Pool<Postgres>,
    pub redis_conn: r2d2Pool<Client>,
}

pub fn init_openapi_route(
    app_state: Arc<AppState>,
    config: &Config,
) -> CorsEndpoint<AddDataEndpoint<Route, Arc<AppState>>> {
    let prefix = config.prefix.clone().unwrap_or("/".to_string());
    let openapi_route = OpenApiService::new(
        (
            ApiAuth,
            ApiUser,
            ApiRole,
            ApiGroup,
            ApiPermission,
            ApiPermissionAttribute,
            ApiRolePermission,
            ApiGroupPermission,
            ApiUserPermission,
        ),
        "Core",
        "1.0",
    )
    .server(prefix.clone());
    let openapi_json_endpoint = openapi_route.spec_endpoint();
    let ui = openapi_route.swagger_ui();
    Route::new()
        .nest(prefix, openapi_route)
        .nest("/docs", ui)
        .at("openapi.json", openapi_json_endpoint)
        .with(AddData::new(app_state))
        .with(Cors::new())
}
