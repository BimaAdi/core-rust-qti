use poem_openapi::{payload::Json, ApiResponse, Object};
use serde::{Deserialize, Serialize};

use super::common::{
    BadRequestResponse, InternalServerErrorResponse, NotFoundResponse, PaginateResponse,
    UnauthorizedResponse,
};

#[derive(Object, Deserialize, Serialize)]
pub struct DetailGroupGroupPermission {
    pub id: String,
    pub group_name: String,
}

#[derive(Object, Deserialize, Serialize)]
pub struct DetailPermissionGroupPermission {
    pub id: String,
    pub permission_name: String,
}

#[derive(Object, Deserialize, Serialize)]
pub struct DetailPermissionAttributeGroupPermission {
    pub id: String,
    pub name: String,
}

#[derive(Object, Deserialize, Serialize)]
pub struct DetailGroupPermission {
    pub group: DetailGroupGroupPermission,
    pub permission: DetailPermissionGroupPermission,
    pub permission_attribute: DetailPermissionAttributeGroupPermission,
}

#[derive(ApiResponse)]
pub enum PaginateGroupPermissionResponses {
    #[oai(status = 200)]
    Ok(Json<PaginateResponse<DetailGroupPermission>>),

    #[oai(status = 400)]
    BadRequest(Json<BadRequestResponse>),

    #[oai(status = 401)]
    Unauthorized(Json<UnauthorizedResponse>),

    #[oai(status = 500)]
    InternalServerError(Json<InternalServerErrorResponse>),
}

#[derive(Object, Deserialize)]
pub struct GroupPermissionCreateRequest {
    pub group_id: String,
    pub permission_id: String,
    pub attribute_id: String,
}

#[derive(Object, Deserialize, Serialize)]
pub struct GroupPermissionCreateResponse {
    pub group_id: String,
    pub permission_id: String,
    pub attribute_id: String,
}

#[derive(ApiResponse)]
pub enum CreateGroupPermissionResponses {
    #[oai(status = 201)]
    Ok(Json<GroupPermissionCreateResponse>),

    #[oai(status = 400)]
    BadRequest(Json<BadRequestResponse>),

    #[oai(status = 401)]
    Unauthorized(Json<UnauthorizedResponse>),

    #[oai(status = 500)]
    InternalServerError(Json<InternalServerErrorResponse>),
}

#[derive(ApiResponse)]
pub enum DeleteGroupPermissionResponses {
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
