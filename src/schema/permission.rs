use poem_openapi::{payload::Json, ApiResponse, Object};
use serde::{Deserialize, Serialize};

use super::common::{
    BadRequestResponse, InternalServerErrorResponse, NotFoundResponse, PaginateResponse,
    UnauthorizedResponse,
};

#[derive(Object, Deserialize, Serialize)]
pub struct DetailUserPermission {
    pub id: String,
    pub user_name: String,
}

#[derive(Object, Deserialize, Serialize)]
pub struct DetailPermission {
    pub id: String,
    pub permission_name: String,
    pub description: String,
    pub is_user: bool,
    pub is_role: bool,
    pub is_group: bool,
    pub created_date: String,
    pub updated_date: String,
    pub created_by: Option<DetailUserPermission>,
    pub updated_by: Option<DetailUserPermission>,
}

#[derive(ApiResponse)]
pub enum PaginatePermissionResponses {
    #[oai(status = 200)]
    Ok(Json<PaginateResponse<DetailPermission>>),

    #[oai(status = 401)]
    Unauthorized(Json<UnauthorizedResponse>),

    #[oai(status = 500)]
    InternalServerError(Json<InternalServerErrorResponse>),
}

#[derive(Object, Deserialize, Serialize)]
pub struct PermissionAllResponse {
    pub id: String,
    pub permission_name: String,
    pub description: String,
    pub is_user: bool,
    pub is_role: bool,
    pub is_group: bool,
    pub created_date: String,
    pub updated_date: String,
}

#[derive(ApiResponse)]
pub enum AllPermissionResponses {
    #[oai(status = 200)]
    Ok(Json<Vec<PermissionAllResponse>>),

    #[oai(status = 401)]
    Unauthorized(Json<UnauthorizedResponse>),

    #[oai(status = 500)]
    InternalServerError(Json<InternalServerErrorResponse>),
}

#[derive(Object, Deserialize, Serialize)]
pub struct PermissionDropdownResponse {
    id: String,
    permission_name: String,
}

#[derive(ApiResponse)]
pub enum DropdownPermissionResponses {
    #[oai(status = 200)]
    Ok(Json<Vec<PermissionDropdownResponse>>),

    #[oai(status = 401)]
    Unauthorized(Json<UnauthorizedResponse>),

    #[oai(status = 500)]
    InternalServerError(Json<InternalServerErrorResponse>),
}

#[derive(Object, Deserialize, Serialize)]
pub struct PermissionAttributeList {
    pub id: String,
    pub name: String,
    pub description: String,
}

#[derive(Object, Deserialize, Serialize)]
pub struct PermissionDetailResponse {
    pub id: String,
    pub permission_name: String,
    pub description: String,
    pub is_user: bool,
    pub is_role: bool,
    pub is_group: bool,
    pub created_date: String,
    pub updated_date: String,
    pub created_by: Option<DetailUserPermission>,
    pub updated_by: Option<DetailUserPermission>,
    pub permission_attribute_ids: Vec<PermissionAttributeList>,
}

#[allow(clippy::large_enum_variant)]
#[derive(ApiResponse)]
pub enum PermissionDetailResponses {
    #[oai(status = 200)]
    Ok(Json<PermissionDetailResponse>),

    #[oai(status = 401)]
    Unauthorized(Json<UnauthorizedResponse>),

    #[oai(status = 404)]
    NotFound(Json<NotFoundResponse>),

    #[oai(status = 500)]
    InternalServerError(Json<InternalServerErrorResponse>),
}

#[derive(Object, Deserialize)]
pub struct PermissionCreateRequest {
    pub permission_name: String,
    pub description: Option<String>,
    pub is_user: bool,
    pub is_role: bool,
    pub is_group: bool,
    pub permission_attribute_ids: Vec<String>,
}

#[derive(Object, Deserialize, Serialize)]
pub struct PermissionCreateResponse {
    pub id: String,
    pub permission_name: String,
    pub description: Option<String>,
    pub is_user: bool,
    pub is_role: bool,
    pub is_group: bool,
}

#[derive(ApiResponse)]
pub enum PermissionCreateResponses {
    #[oai(status = 201)]
    Created(Json<PermissionCreateResponse>),

    #[oai(status = 400)]
    BadRequest(Json<BadRequestResponse>),

    #[oai(status = 401)]
    Unauthorized(Json<UnauthorizedResponse>),

    #[oai(status = 500)]
    InternalServerError(Json<InternalServerErrorResponse>),
}

#[derive(Object, Deserialize)]
pub struct PermissionUpdateRequest {
    pub permission_name: String,
    pub description: String,
    pub is_user: bool,
    pub is_role: bool,
    pub is_group: bool,
    pub permission_attribute_ids: Vec<String>,
}

#[derive(Object, Deserialize, Serialize)]
pub struct PermissionUpdateResponse {
    pub id: String,
    pub permission_name: String,
    pub description: String,
    pub is_user: bool,
    pub is_role: bool,
    pub is_group: bool,
}

#[derive(ApiResponse)]
pub enum PermissionUpdateResponses {
    #[oai(status = 201)]
    Ok(Json<PermissionUpdateResponse>),

    #[oai(status = 400)]
    BadRequest(Json<BadRequestResponse>),

    #[oai(status = 401)]
    Unauthorized(Json<UnauthorizedResponse>),

    #[oai(status = 404)]
    NotFound(Json<NotFoundResponse>),

    #[oai(status = 500)]
    InternalServerError(Json<InternalServerErrorResponse>),
}

#[derive(ApiResponse)]
pub enum PermissionDeleteResponses {
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
