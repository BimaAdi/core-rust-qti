use poem_openapi::{payload::Json, ApiResponse, Object};
use serde::{Deserialize, Serialize};

use super::common::{
    BadRequestResponse, InternalServerErrorResponse, NotFoundResponse, PaginateResponse,
    UnauthorizedResponse,
};

#[derive(Object, Deserialize, Serialize)]
pub struct DetailPermissionAttribute {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
}

#[derive(ApiResponse)]
pub enum PaginatePermissionAttributeResponses {
    #[oai(status = 200)]
    Ok(Json<PaginateResponse<DetailPermissionAttribute>>),

    #[oai(status = 401)]
    Unauthorized(Json<UnauthorizedResponse>),

    #[oai(status = 500)]
    InternalServerError(Json<InternalServerErrorResponse>),
}

#[derive(ApiResponse)]
pub enum DropdownPermissionAttributeResponses {
    #[oai(status = 200)]
    Ok(Json<Vec<DetailPermissionAttribute>>),

    #[oai(status = 401)]
    Unauthorized(Json<UnauthorizedResponse>),

    #[oai(status = 500)]
    InternalServerError(Json<InternalServerErrorResponse>),
}

#[derive(ApiResponse)]
pub enum DetailPermissionAttributeResponses {
    #[oai(status = 200)]
    Ok(Json<DetailPermissionAttribute>),

    #[oai(status = 401)]
    Unauthorized(Json<UnauthorizedResponse>),

    #[oai(status = 404)]
    NotFound(Json<NotFoundResponse>),

    #[oai(status = 500)]
    InternalServerError(Json<InternalServerErrorResponse>),
}

#[derive(Object, Deserialize)]
pub struct CreatePermissionAttributeRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(ApiResponse)]
pub enum CreatePermissionAttributeResponses {
    #[oai(status = 201)]
    Ok(Json<DetailPermissionAttribute>),

    #[oai(status = 400)]
    BadRequest(Json<BadRequestResponse>),

    #[oai(status = 401)]
    Unauthorized(Json<UnauthorizedResponse>),

    #[oai(status = 500)]
    InternalServerError(Json<InternalServerErrorResponse>),
}

#[derive(Object, Deserialize)]
pub struct UpdatePermissionAttributeRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(ApiResponse)]
pub enum UpdatePermissionAttributeResponses {
    #[oai(status = 200)]
    Ok(Json<DetailPermissionAttribute>),

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
pub enum DeletePermissionAttributeResponses {
    #[oai(status = 204)]
    NoContent,

    #[oai(status = 401)]
    Unauthorized(Json<UnauthorizedResponse>),

    #[oai(status = 404)]
    NotFound(Json<NotFoundResponse>),

    #[oai(status = 500)]
    InternalServerError(Json<InternalServerErrorResponse>),
}
