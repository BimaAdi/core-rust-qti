use poem_openapi::{payload::Json, ApiResponse, Object};
use serde::{Deserialize, Serialize};

use super::common::{
    BadRequestResponse, InternalServerErrorResponse, NotFoundResponse, PaginateResponse,
    UnauthorizedResponse,
};

#[derive(Object, Deserialize, Serialize)]
pub struct RoleDetailUser {
    pub id: String,
    pub user_name: String,
}

#[derive(Object, Deserialize, Serialize)]
pub struct DetailRolePagination {
    pub id: String,
    pub role_name: String,
    pub description: Option<String>,
    pub is_active: Option<bool>,
    pub created_by: Option<RoleDetailUser>,
    pub updated_by: Option<RoleDetailUser>,
    pub created_date: Option<String>,
    pub updated_date: Option<String>,
}

#[derive(ApiResponse)]
pub enum PaginateRoleResponses {
    #[oai(status = 200)]
    Ok(Json<PaginateResponse<DetailRolePagination>>),

    #[oai(status = 401)]
    Unauthorized(Json<UnauthorizedResponse>),

    #[oai(status = 500)]
    InternalServerError(Json<InternalServerErrorResponse>),
}

#[derive(Object, Deserialize, Serialize)]
pub struct RoleAllResponse {
    pub id: String,
    pub role_name: String,
    pub description: Option<String>,
    pub is_active: Option<bool>,
    pub created_date: Option<String>,
    pub updated_date: Option<String>,
    pub created_by: Option<RoleDetailUser>,
    pub updated_by: Option<RoleDetailUser>,
}

#[derive(ApiResponse)]
pub enum RoleAllResponses {
    #[oai(status = 200)]
    Ok(Json<Vec<RoleAllResponse>>),

    #[oai(status = 401)]
    Unauthorized(Json<UnauthorizedResponse>),

    #[oai(status = 500)]
    InternalServerError(Json<InternalServerErrorResponse>),
}

#[derive(Object, Deserialize)]
pub struct RoleDropdownResponse {
    pub id: String,
    pub role_name: String,
}

#[derive(ApiResponse)]
pub enum RoleDropdownResponses {
    #[oai(status = 200)]
    Ok(Json<Vec<RoleDropdownResponse>>),

    #[oai(status = 401)]
    Unauthorized(Json<UnauthorizedResponse>),

    #[oai(status = 500)]
    InternalServerError(Json<InternalServerErrorResponse>),
}

#[derive(Object, Deserialize)]
pub struct RoleDetailSuccessResponse {
    pub id: String,
    pub role_name: String,
    pub description: Option<String>,
    pub is_active: Option<bool>,
    pub created_date: Option<String>,
    pub updated_date: Option<String>,
    pub created_by: Option<RoleDetailUser>,
    pub updated_by: Option<RoleDetailUser>,
}

#[derive(ApiResponse)]
pub enum RoleDetailResponses {
    #[oai(status = 200)]
    Ok(Json<RoleDetailSuccessResponse>),

    #[oai(status = 400)]
    BadRequest(Json<BadRequestResponse>),

    #[oai(status = 401)]
    Unauthorized(Json<UnauthorizedResponse>),

    #[oai(status = 404)]
    NotFound(Json<NotFoundResponse>),

    #[oai(status = 500)]
    InternalServerError(Json<InternalServerErrorResponse>),
}

#[derive(Object, Deserialize)]
pub struct RoleCreateRequest {
    pub role_name: String,
    pub description: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Object, Deserialize)]
pub struct RoleCreateResponse {
    pub id: String,
    pub role_name: String,
    pub description: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(ApiResponse)]
pub enum RoleCreateResponses {
    #[oai(status = 201)]
    Ok(Json<RoleCreateResponse>),

    #[oai(status = 401)]
    Unauthorized(Json<UnauthorizedResponse>),

    #[oai(status = 500)]
    InternalServerError(Json<InternalServerErrorResponse>),
}

#[derive(Object, Deserialize)]
pub struct RoleUpdateRequest {
    pub role_name: String,
    pub description: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Object, Deserialize)]
pub struct RoleUpdateResponse {
    pub id: String,
    pub role_name: String,
    pub description: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(ApiResponse)]
pub enum RoleUpdateResponses {
    #[oai(status = 200)]
    Ok(Json<RoleUpdateResponse>),

    #[oai(status = 401)]
    Unauthorized(Json<UnauthorizedResponse>),

    #[oai(status = 404)]
    NotFound(Json<NotFoundResponse>),

    #[oai(status = 500)]
    InternalServerError(Json<InternalServerErrorResponse>),
}

#[derive(ApiResponse)]
pub enum RoleDeleteResponses {
    #[oai(status = 204)]
    NoContent,

    #[oai(status = 401)]
    Unauthorized(Json<UnauthorizedResponse>),

    #[oai(status = 404)]
    NotFound(Json<NotFoundResponse>),

    #[oai(status = 500)]
    InternalServerError(Json<InternalServerErrorResponse>),
}
