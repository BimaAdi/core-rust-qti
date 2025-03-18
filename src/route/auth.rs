use std::sync::Arc;

use chrono::{Duration, FixedOffset, Local};
use poem::web::Data;
use poem_openapi::{payload::Json, OpenApi, Tags};

use crate::{
    core::{
        security::{
            generate_refresh_token_from_user, generate_token_from_user, verify_hash_password,
        },
        session::add_session,
    },
    repository::user::get_user_by_username,
    schema::{
        auth::{LoginRequest, LoginResponse, LoginResponses},
        common::{BadRequestResponse, InternalServerErrorResponse},
    },
    settings::get_config,
    AppState,
};

#[derive(Tags)]
enum ApiAuthTags {
    Auth,
}

pub struct ApiAuth;

#[OpenApi]
impl ApiAuth {
    #[oai(path = "/auth/login", method = "post", tag = "ApiAuthTags::Auth")]
    async fn auth_login(
        &self,
        json: Json<LoginRequest>,
        state: Data<&Arc<AppState>>,
    ) -> LoginResponses {
        // Begin db transaction
        let mut tx = match state.db.begin().await {
            Ok(val) => val,
            Err(err) => {
                return LoginResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.auth",
                        "auth_login",
                        "begin transaction",
                        &err.to_string(),
                    ),
                ));
            }
        };

        // get redis conn from pool
        let mut redis_conn = match state.redis_conn.get() {
            Ok(val) => val,
            Err(err) => {
                return LoginResponses::InternalServerError(Json(InternalServerErrorResponse::new(
                    "route.auth",
                    "auth_login",
                    "get redis pool connection",
                    &err.to_string(),
                )))
            }
        };

        // get usename on db
        let (user, user_profile) = match get_user_by_username(&mut tx, &json.user_name).await {
            Ok(val) => val,
            Err(err) => {
                return LoginResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.auth",
                        "auth_login",
                        "check user on database",
                        &err.to_string(),
                    ),
                ));
            }
        };
        if user.is_none() || user_profile.is_none() {
            return LoginResponses::BadRequet(Json(BadRequestResponse {
                message: "Invalid credentials".to_string(),
            }));
        }
        let user = user.unwrap();
        // let user_profile = user_profile.unwrap();

        // validate user password
        let is_valid = match verify_hash_password(&json.password, &user.password) {
            Ok(val) => val,
            Err(err) => {
                return LoginResponses::InternalServerError(Json(InternalServerErrorResponse::new(
                    "route.auth",
                    "auth_login",
                    "validate user password",
                    &err.to_string(),
                )))
            }
        };
        if !is_valid {
            return LoginResponses::BadRequet(Json(BadRequestResponse {
                message: "Invalid credentials".to_string(),
            }));
        }

        let config = get_config();
        let token = match generate_token_from_user(user.clone(), config.clone()).await {
            Ok(val) => val,
            Err(err) => {
                return LoginResponses::InternalServerError(Json(InternalServerErrorResponse::new(
                    "route.auth",
                    "auth_login",
                    "generate token",
                    &err.to_string(),
                )))
            }
        };

        let refresh_token = match generate_refresh_token_from_user(user.clone(), config.clone())
            .await
        {
            Ok(val) => val,
            Err(err) => {
                return LoginResponses::InternalServerError(Json(InternalServerErrorResponse::new(
                    "route.auth",
                    "auth_login",
                    "generate refresh token",
                    &err.to_string(),
                )))
            }
        };

        if let Err(err) = add_session(
            &mut redis_conn,
            &user,
            &config,
            token.clone(),
            refresh_token.clone(),
        ) {
            return LoginResponses::InternalServerError(Json(InternalServerErrorResponse::new(
                "route.auth",
                "auth_login",
                "add_session to redis",
                &err.to_string(),
            )));
        }
        let now = Local::now();
        let exp = now + Duration::minutes(config.jwt_exp as i64);
        let exp_refresh_token = now + Duration::minutes(config.jwt_refresh_exp as i64);
        let offset = FixedOffset::east_opt(7 * 60 * 60).unwrap(); // +0700
        LoginResponses::Ok(Json(LoginResponse {
            exp: exp
                .with_timezone(&offset)
                .format("%Y-%m-%d %H:%M:%S")
                .to_string(),
            exp_in: now.timestamp() as i32 + config.jwt_exp as i32,
            exp_refresh_token: exp_refresh_token
                .with_timezone(&offset)
                .format("%Y-%m-%d %H:%M:%S")
                .to_string(),
            refresh_token,
            token,
            token_type: "Bearer".to_string(),
        }))
    }
}
