use poem_openapi::{payload::Json, ApiResponse, Object};
use serde::{Deserialize, Serialize};

use super::common::{
    BadRequestResponse, InternalServerErrorResponse, NotFoundResponse, PaginateResponse,
    UnauthorizedResponse,
};

#[derive(Object, Deserialize, Serialize)]
pub struct GroupDetailUser {
    pub id: String,
    pub user_name: String,
}

#[derive(Object, Deserialize, Serialize)]
pub struct DetailGroupPagination {
    pub id: String,
    pub group_name: String,
    pub description: Option<String>,
    pub is_active: Option<bool>,
    pub created_by: Option<GroupDetailUser>,
    pub updated_by: Option<GroupDetailUser>,
    pub created_date: Option<String>,
    pub updated_date: Option<String>,
}

#[derive(ApiResponse)]
pub enum PaginateGroupResponses {
    #[oai(status = 200)]
    Ok(Json<PaginateResponse<DetailGroupPagination>>),

    #[oai(status = 401)]
    Unauthorized(Json<UnauthorizedResponse>),

    #[oai(status = 500)]
    InternalServerError(Json<InternalServerErrorResponse>),
}

#[derive(Object, Deserialize, Serialize)]
pub struct GroupAllResponse {
    pub id: String,
    pub group_name: String,
    pub description: Option<String>,
    pub is_active: Option<bool>,
    pub created_date: Option<String>,
    pub updated_date: Option<String>,
    pub created_by: Option<GroupDetailUser>,
    pub updated_by: Option<GroupDetailUser>,
}

#[derive(ApiResponse)]
pub enum GroupAllResponses {
    #[oai(status = 200)]
    Ok(Json<Vec<GroupAllResponse>>),

    #[oai(status = 401)]
    Unauthorized(Json<UnauthorizedResponse>),

    #[oai(status = 500)]
    InternalServerError(Json<InternalServerErrorResponse>),
}

#[derive(Object, Deserialize)]
pub struct GroupDropdownResponse {
    pub id: String,
    pub group_name: String,
}

#[derive(ApiResponse)]
pub enum GroupDropdownResponses {
    #[oai(status = 200)]
    Ok(Json<Vec<GroupDropdownResponse>>),

    #[oai(status = 401)]
    Unauthorized(Json<UnauthorizedResponse>),

    #[oai(status = 500)]
    InternalServerError(Json<InternalServerErrorResponse>),
}

#[derive(Object, Deserialize)]
pub struct GroupDetailSuccessResponse {
    pub id: String,
    pub group_name: String,
    pub description: Option<String>,
    pub is_active: Option<bool>,
    pub created_date: Option<String>,
    pub updated_date: Option<String>,
    pub created_by: Option<GroupDetailUser>,
    pub updated_by: Option<GroupDetailUser>,
}

#[derive(ApiResponse)]
pub enum GroupDetailResponses {
    #[oai(status = 200)]
    Ok(Json<GroupDetailSuccessResponse>),

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
pub struct GroupCreateRequest {
    pub group_name: String,
    pub description: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Object, Deserialize)]
pub struct GroupCreateResponse {
    pub id: String,
    pub group_name: String,
    pub description: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(ApiResponse)]
pub enum GroupCreateResponses {
    #[oai(status = 201)]
    Ok(Json<GroupCreateResponse>),

    #[oai(status = 401)]
    Unauthorized(Json<UnauthorizedResponse>),

    #[oai(status = 500)]
    InternalServerError(Json<InternalServerErrorResponse>),
}

#[derive(Object, Deserialize)]
pub struct GroupUpdateRequest {
    pub group_name: String,
    pub description: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Object, Deserialize)]
pub struct GroupUpdateResponse {
    pub id: String,
    pub group_name: String,
    pub description: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(ApiResponse)]
pub enum GroupUpdateResponses {
    #[oai(status = 200)]
    Ok(Json<GroupUpdateResponse>),

    #[oai(status = 401)]
    Unauthorized(Json<UnauthorizedResponse>),

    #[oai(status = 404)]
    NotFound(Json<NotFoundResponse>),

    #[oai(status = 500)]
    InternalServerError(Json<InternalServerErrorResponse>),
}

#[derive(ApiResponse)]
pub enum GroupDeleteResponses {
    #[oai(status = 204)]
    NoContent,

    #[oai(status = 401)]
    Unauthorized(Json<UnauthorizedResponse>),

    #[oai(status = 404)]
    NotFound(Json<NotFoundResponse>),

    #[oai(status = 500)]
    InternalServerError(Json<InternalServerErrorResponse>),
}
