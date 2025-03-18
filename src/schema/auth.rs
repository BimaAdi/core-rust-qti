use poem_openapi::{payload::Json, ApiResponse, Object};
use serde::Deserialize;

use crate::schema::common::{BadRequestResponse, InternalServerErrorResponse};

#[derive(Object, Deserialize)]
pub struct LoginRequest {
    pub user_name: String,
    pub password: String,
}

#[derive(Object, Deserialize)]
pub struct LoginResponse {
    pub exp: String,
    pub exp_in: i32,
    pub exp_refresh_token: String,
    pub refresh_token: String,
    pub token: String,
    pub token_type: String,
}

#[derive(ApiResponse)]
pub enum LoginResponses {
    #[oai(status = 200)]
    Ok(Json<LoginResponse>),

    #[oai(status = 400)]
    BadRequet(Json<BadRequestResponse>),

    #[oai(status = 500)]
    InternalServerError(Json<InternalServerErrorResponse>),
}
