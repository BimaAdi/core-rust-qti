use poem_openapi::{payload::Json, ApiResponse, Object};
use serde::Deserialize;

use super::common::{
    BadRequestResponse, ForbiddenResponse, InternalServerErrorResponse, NotFoundResponse,
    PaginateResponse, UnauthorizedResponse,
};

#[derive(Object, Deserialize)]
pub struct DetailCreatedOrUpdatedUser {
    pub id: String,
    pub user_name: String,
}

#[derive(Object, Deserialize)]
pub struct DetailUser {
    pub id: String,
    pub user_name: String,
    pub is_active: bool,
    pub is_2faenabled: bool,
    pub created_date: String,
    pub updated_date: String,
    pub created_by: Option<DetailCreatedOrUpdatedUser>,
}

#[derive(ApiResponse)]
pub enum GetPaginateUserResponses {
    #[oai(status = 200)]
    Ok(Json<PaginateResponse<DetailUser>>),

    #[oai(status = 401)]
    Unauthorized(Json<UnauthorizedResponse>),

    #[oai(status = 403)]
    Forbidden(Json<ForbiddenResponse>),

    #[oai(status = 500)]
    InternalServerError(Json<InternalServerErrorResponse>),
}

#[derive(ApiResponse)]
pub enum GetAllUserResponses {
    #[oai(status = 200)]
    Ok(Json<PaginateResponse<DetailUser>>),

    #[oai(status = 401)]
    Unauthorized(Json<UnauthorizedResponse>),

    #[oai(status = 500)]
    InternalServerError(Json<InternalServerErrorResponse>),
}

#[derive(Object, Deserialize)]
pub struct DetailUserProfile {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub address: Option<String>,
}

#[derive(Object, Deserialize)]
pub struct DetailRole {
    pub id: String,
    pub role_name: String,
}

#[derive(Object, Deserialize)]
pub struct DetailGroup {
    pub id: String,
    pub group_name: String,
}

#[derive(Object, Deserialize)]
pub struct DetailGroupRole {
    pub role: Option<DetailRole>,
    pub group: Option<DetailGroup>,
}

#[derive(Object, Deserialize)]
pub struct UserDetailResponse {
    pub id: String,
    pub user_name: String,
    pub is_active: Option<bool>,
    pub is_2faenabled: Option<bool>,
    pub created_date: Option<String>,
    pub updated_date: Option<String>,
    pub user_profile: Option<DetailUserProfile>,
    pub created_by: Option<DetailCreatedOrUpdatedUser>,
    pub updated_by: Option<DetailCreatedOrUpdatedUser>,
    pub group_roles: Vec<DetailGroupRole>,
}

#[allow(clippy::large_enum_variant)]
#[derive(ApiResponse)]
pub enum UserDetailResponses {
    #[oai(status = 200)]
    Ok(Json<UserDetailResponse>),

    #[oai(status = 401)]
    Unauthorized(Json<UnauthorizedResponse>),

    #[oai(status = 403)]
    Forbidden(Json<ForbiddenResponse>),

    #[oai(status = 404)]
    NotFound(Json<NotFoundResponse>),

    #[oai(status = 500)]
    InternalServerError(Json<InternalServerErrorResponse>),
}

#[derive(Object, Deserialize)]
pub struct GroupRole {
    pub group_id: String,
    pub role_id: String,
}

#[derive(Object, Deserialize)]
pub struct UserCreateRequest {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub is_active: bool,
    pub password: String,
    pub user_name: String,
    pub address: Option<String>,
    pub group_roles: Option<Vec<GroupRole>>,
}

#[derive(Object, Deserialize)]
pub struct UserCreateResponse {
    pub id: String,
    pub user_name: String,
    pub is_active: Option<bool>,
    pub group_roles: Vec<DetailGroupRole>,
    pub user_profile: Option<DetailUserProfile>,
}

#[derive(ApiResponse)]
pub enum UserCreateResponses {
    #[oai(status = 201)]
    Created(Json<UserCreateResponse>),

    #[oai(status = 400)]
    BadRequest(Json<BadRequestResponse>),

    #[oai(status = 401)]
    Unauthorized(Json<UnauthorizedResponse>),

    #[oai(status = 403)]
    Forbidden(Json<ForbiddenResponse>),

    #[oai(status = 500)]
    InternalServerError(Json<InternalServerErrorResponse>),
}

#[derive(Object, Deserialize)]
pub struct UserUpdateRequest {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub is_active: bool,
    pub password: String,
    pub user_name: String,
    pub address: Option<String>,
    pub group_roles: Option<Vec<GroupRole>>,
}

#[derive(Object, Deserialize)]
pub struct UserUpdateResponse {
    pub id: String,
    pub user_name: String,
    pub is_active: bool,
    pub group_roles: Vec<DetailGroupRole>,
    pub user_profile: Option<DetailUserProfile>,
}

#[derive(ApiResponse)]
pub enum UserUpdateResponses {
    #[oai(status = 200)]
    Created(Json<UserUpdateResponse>),

    #[oai(status = 400)]
    BadRequest(Json<BadRequestResponse>),

    #[oai(status = 401)]
    Unauthorized(Json<UnauthorizedResponse>),

    #[oai(status = 403)]
    Forbidden(Json<ForbiddenResponse>),

    #[oai(status = 404)]
    NotFound(Json<NotFoundResponse>),

    #[oai(status = 500)]
    InternalServerError(Json<InternalServerErrorResponse>),
}

#[derive(ApiResponse)]
pub enum UserDeleteResponses {
    #[oai(status = 204)]
    NoContent,

    #[oai(status = 401)]
    Unauthorized(Json<UnauthorizedResponse>),

    #[oai(status = 403)]
    Forbidden(Json<ForbiddenResponse>),

    #[oai(status = 404)]
    NotFound(Json<NotFoundResponse>),

    #[oai(status = 500)]
    InternalServerError(Json<InternalServerErrorResponse>),
}

#[derive(Object, Deserialize)]
pub struct ResetPasswordRequest {
    pub new_password: String,
    pub confirm_new_password: String,
}

#[derive(Object, Deserialize)]
pub struct ResetPasswordResponse {
    message: String,
}

#[derive(ApiResponse)]
pub enum ResetPasswordResponses {
    #[oai(status = 200)]
    Ok(Json<ResetPasswordResponse>),

    #[oai(status = 400)]
    BadRequest(Json<BadRequestResponse>),

    #[oai(status = 401)]
    Unauthorized(Json<UnauthorizedResponse>),

    #[oai(status = 403)]
    Forbidden(Json<ForbiddenResponse>),

    #[oai(status = 500)]
    InternalServerError(Json<InternalServerErrorResponse>),
}

#[derive(Object, Deserialize)]
pub struct ChangeStatusRequest {
    pub status: bool,
}

#[derive(ApiResponse)]
pub enum ChangeStatusResponses {
    #[oai(status = 204)]
    NoContent,

    #[oai(status = 401)]
    Unauthorized(Json<UnauthorizedResponse>),

    #[oai(status = 403)]
    Forbidden(Json<ForbiddenResponse>),

    #[oai(status = 404)]
    NotFound(Json<NotFoundResponse>),

    #[oai(status = 500)]
    InternalServerError(Json<InternalServerErrorResponse>),
}

#[derive(Object, Deserialize)]
pub struct AddUserGroupRoleRequest {
    pub user_id: String,
    pub role_id: String,
    pub group_id: String,
}

#[derive(Object, Deserialize)]
pub struct AddUserGroupRoleResponse {
    pub id: String,
    pub user_id: String,
    pub role_id: String,
    pub group_id: String,
}

#[derive(ApiResponse)]
pub enum AddUserGroupRoleResponses {
    #[oai(status = 201)]
    Created(Json<AddUserGroupRoleResponse>),

    #[oai(status = 401)]
    Unauthorized(Json<UnauthorizedResponse>),

    #[oai(status = 403)]
    Forbidden(Json<ForbiddenResponse>),

    #[oai(status = 404)]
    NotFound(Json<NotFoundResponse>),

    #[oai(status = 500)]
    InternalServerError(Json<InternalServerErrorResponse>),
}

#[derive(ApiResponse)]
pub enum DeleteUserGroupRoleResponses {
    #[oai(status = 204)]
    NoContent,

    #[oai(status = 401)]
    Unauthorized(Json<UnauthorizedResponse>),

    #[oai(status = 403)]
    Forbidden(Json<ForbiddenResponse>),

    #[oai(status = 404)]
    NotFound(Json<NotFoundResponse>),

    #[oai(status = 500)]
    InternalServerError(Json<InternalServerErrorResponse>),
}
