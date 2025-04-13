use poem_openapi::{payload::Json, ApiResponse, Object};
use serde::{Deserialize, Serialize};

use super::common::{
    BadRequestResponse, InternalServerErrorResponse, NotFoundResponse, PaginateResponse,
    UnauthorizedResponse,
};

#[derive(Object, Deserialize, Serialize)]
pub struct DetailUserUserPermission {
    pub id: String,
    pub user_name: String,
}

#[derive(Object, Deserialize, Serialize)]
pub struct DetailPermissionUserPermission {
    pub id: String,
    pub permission_name: String,
}

#[derive(Object, Deserialize, Serialize)]
pub struct DetailPermissionAttributeUserPermission {
    pub id: String,
    pub name: String,
}

#[derive(Object, Deserialize, Serialize)]
pub struct DetailUserPermissionResponse {
    pub user: DetailUserUserPermission,
    pub permission: DetailPermissionUserPermission,
    pub permission_attribute: DetailPermissionAttributeUserPermission,
}

#[derive(ApiResponse)]
pub enum PaginateUserPermissionResponses {
    #[oai(status = 200)]
    Ok(Json<PaginateResponse<DetailUserPermissionResponse>>),

    #[oai(status = 400)]
    BadRequest(Json<BadRequestResponse>),

    #[oai(status = 401)]
    Unauthorized(Json<UnauthorizedResponse>),

    #[oai(status = 500)]
    InternalServerError(Json<InternalServerErrorResponse>),
}

#[derive(Object, Deserialize)]
pub struct UserPermissionCreateRequest {
    pub user_id: String,
    pub permission_id: String,
    pub attribute_id: String,
}

#[derive(Object, Deserialize, Serialize)]
pub struct UserPermissionCreateResponse {
    pub user_id: String,
    pub permission_id: String,
    pub attribute_id: String,
}

#[derive(ApiResponse)]
pub enum CreateUserPermissionResponses {
    #[oai(status = 201)]
    Ok(Json<UserPermissionCreateResponse>),

    #[oai(status = 400)]
    BadRequest(Json<BadRequestResponse>),

    #[oai(status = 401)]
    Unauthorized(Json<UnauthorizedResponse>),

    #[oai(status = 500)]
    InternalServerError(Json<InternalServerErrorResponse>),
}

#[derive(ApiResponse)]
pub enum DeleteUserPermissionResponses {
    #[oai(status = 204)]
    NoContent,

    #[oai(status = 400)]
    BadRequest(Json<BadRequestResponse>),

    #[oai(status = 401)]
    Unauthorized(Json<UnauthorizedResponse>),

    #[oai(status = 404)]
    NotFound(Json<NotFoundResponse>),

    #[oai(status = 500)]
    InternalServerError(Json<InternalServerErrorResponse>),
}
