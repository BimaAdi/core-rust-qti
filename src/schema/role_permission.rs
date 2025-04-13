use poem_openapi::{payload::Json, ApiResponse, Object};
use serde::{Deserialize, Serialize};

use super::common::{
    BadRequestResponse, InternalServerErrorResponse, NotFoundResponse, PaginateResponse,
    UnauthorizedResponse,
};

#[derive(Object, Deserialize, Serialize)]
pub struct DetailRoleRolePermission {
    pub id: String,
    pub role_name: String,
}

#[derive(Object, Deserialize, Serialize)]
pub struct DetailPermissionRolePermission {
    pub id: String,
    pub permission_name: String,
}

#[derive(Object, Deserialize, Serialize)]
pub struct DetailPermissionAttributeRolePermission {
    pub id: String,
    pub name: String,
}

#[derive(Object, Deserialize, Serialize)]
pub struct DetailRolePermission {
    pub role: DetailRoleRolePermission,
    pub permission: DetailPermissionRolePermission,
    pub permission_attribute: DetailPermissionAttributeRolePermission,
}

#[derive(ApiResponse)]
pub enum PaginateRolePermissionResponses {
    #[oai(status = 200)]
    Ok(Json<PaginateResponse<DetailRolePermission>>),

    #[oai(status = 400)]
    BadRequest(Json<BadRequestResponse>),

    #[oai(status = 401)]
    Unauthorized(Json<UnauthorizedResponse>),

    #[oai(status = 500)]
    InternalServerError(Json<InternalServerErrorResponse>),
}

#[derive(Object, Deserialize)]
pub struct RolePermissionCreateRequest {
    pub role_id: String,
    pub permission_id: String,
    pub attribute_id: String,
}

#[derive(Object, Deserialize, Serialize)]
pub struct RolePermissionCreateResponse {
    pub role_id: String,
    pub permission_id: String,
    pub attribute_id: String,
}

#[derive(ApiResponse)]
pub enum CreateRolePermissionResponses {
    #[oai(status = 201)]
    Ok(Json<RolePermissionCreateResponse>),

    #[oai(status = 400)]
    BadRequest(Json<BadRequestResponse>),

    #[oai(status = 401)]
    Unauthorized(Json<UnauthorizedResponse>),

    #[oai(status = 500)]
    InternalServerError(Json<InternalServerErrorResponse>),
}

#[derive(ApiResponse)]
pub enum DeleteRolePermissionResponses {
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
