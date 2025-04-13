use std::sync::Arc;

use chrono::Local;
use poem::web::Data;
use poem_openapi::{param::Query, payload::Json, OpenApi, Tags};
use uuid::Uuid;

use crate::{
    core::security::{get_user_from_token, BearerAuthorization},
    model::role_permission::RolePermission,
    repository::{
        permission::get_permission_by_id,
        permission_attribute::get_permission_attribute_by_id,
        role::get_role_by_id,
        role_permission::{
            create_role_permission, delete_role_permission, get_all_role_permission,
            get_detail_role_permission,
        },
    },
    schema::{
        common::{
            BadRequestResponse, InternalServerErrorResponse, NotFoundResponse, PaginateResponse,
            UnauthorizedResponse,
        },
        role_permission::{
            CreateRolePermissionResponses, DeleteRolePermissionResponses,
            DetailPermissionAttributeRolePermission, DetailPermissionRolePermission,
            DetailRolePermission, DetailRoleRolePermission, PaginateRolePermissionResponses,
            RolePermissionCreateRequest, RolePermissionCreateResponse,
        },
    },
    AppState,
};

#[derive(Tags)]
enum ApiRolePermissionTags {
    RolePermission,
}

pub struct ApiRolePermission;

#[OpenApi]
impl ApiRolePermission {
    #[oai(
        path = "/role-permissions",
        method = "get",
        tag = "ApiRolePermissionTags::RolePermission"
    )]
    async fn paginate_role_permission_api(
        &self,
        Query(role_id): Query<String>,
        Query(page): Query<Option<u32>>,
        Query(page_size): Query<Option<u32>>,
        Query(all): Query<Option<bool>>,
        state: Data<&Arc<AppState>>,
        auth: BearerAuthorization,
    ) -> PaginateRolePermissionResponses {
        // Begin db transaction
        let mut tx = match state.db.begin().await {
            Ok(val) => val,
            Err(err) => {
                return PaginateRolePermissionResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.role_permission",
                        "paginate_role_permission_api",
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
                return PaginateRolePermissionResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.role_permission",
                        "paginate_role_permission_api",
                        "get redis pool connection",
                        &err.to_string(),
                    ),
                ))
            }
        };

        // Validate user token
        let jwt_token = auth.0.token;
        let request_user =
            match get_user_from_token(&mut tx, &mut redis_conn, jwt_token.clone()).await {
                Ok(val) => val,
                Err(err) => {
                    return PaginateRolePermissionResponses::InternalServerError(Json(
                        InternalServerErrorResponse::new(
                            "route.role_permission",
                            "paginate_role_permission_api",
                            "get user from token",
                            &err.to_string(),
                        ),
                    ))
                }
            };
        if request_user.is_none() {
            return PaginateRolePermissionResponses::Unauthorized(Json(
                UnauthorizedResponse::default(),
            ));
        }

        // Validasi
        let role_id = match Uuid::parse_str(&role_id) {
            Ok(val) => val,
            Err(_) => {
                return PaginateRolePermissionResponses::BadRequest(Json(BadRequestResponse {
                    message: format!("role with id = {} not found", role_id),
                }))
            }
        };
        let role = match get_role_by_id(&mut tx, &role_id).await {
            Ok(val) => val,
            Err(err) => {
                return PaginateRolePermissionResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.role_permission",
                        "paginate_role_permission_api",
                        "get_role_by_id",
                        &err.to_string(),
                    ),
                ))
            }
        };
        if role.is_none() {
            return PaginateRolePermissionResponses::BadRequest(Json(BadRequestResponse {
                message: format!("role with id = {} not found", role_id),
            }));
        }
        let role = role.unwrap();

        let page = page.unwrap_or(1);
        let page_size = page_size.unwrap_or(10);
        let (data, counts, page_count) = match get_all_role_permission(
            &mut tx,
            Some(page),
            Some(page_size),
            &role_id,
            all,
        )
        .await
        {
            Ok(val) => val,
            Err(err) => {
                return PaginateRolePermissionResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.role_permission",
                        "paginate_role_permission_api",
                        "get_all_role_permission",
                        &err.to_string(),
                    ),
                ))
            }
        };

        let mut results: Vec<DetailRolePermission> = vec![];
        for item in data {
            let permission = match get_permission_by_id(&mut tx, &item.permission_id).await {
                Ok(val) => val.unwrap(),
                Err(err) => {
                    return PaginateRolePermissionResponses::InternalServerError(Json(
                        InternalServerErrorResponse::new(
                            "route.role_permission",
                            "paginate_role_permission_api",
                            "get_permission_by_id",
                            &err.to_string(),
                        ),
                    ))
                }
            };
            let attribute = match get_permission_attribute_by_id(&mut tx, &item.attribute_id).await
            {
                Ok(val) => val.unwrap(),
                Err(err) => {
                    return PaginateRolePermissionResponses::InternalServerError(Json(
                        InternalServerErrorResponse::new(
                            "route.role_permission",
                            "paginate_role_permission_api",
                            "get_permission_attribute_by_id",
                            &err.to_string(),
                        ),
                    ))
                }
            };
            results.push(DetailRolePermission {
                role: DetailRoleRolePermission {
                    id: role.id.to_string(),
                    role_name: role.role_name.clone(),
                },
                permission: DetailPermissionRolePermission {
                    id: permission.id.to_string(),
                    permission_name: permission.permission_name,
                },
                permission_attribute: DetailPermissionAttributeRolePermission {
                    id: attribute.id.to_string(),
                    name: attribute.name,
                },
            });
        }
        PaginateRolePermissionResponses::Ok(Json(PaginateResponse {
            counts,
            page,
            page_count,
            page_size,
            results,
        }))
    }

    #[oai(
        path = "/role-permissions",
        method = "post",
        tag = "ApiRolePermissionTags::RolePermission"
    )]
    async fn create_role_permission_api(
        &self,
        Json(json): Json<RolePermissionCreateRequest>,
        state: Data<&Arc<AppState>>,
        auth: BearerAuthorization,
    ) -> CreateRolePermissionResponses {
        // Begin db transaction
        let mut tx = match state.db.begin().await {
            Ok(val) => val,
            Err(err) => {
                return CreateRolePermissionResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.role_permission",
                        "create_role_permission_api",
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
                return CreateRolePermissionResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.role_permission",
                        "create_role_permission_api",
                        "get redis pool connection",
                        &err.to_string(),
                    ),
                ))
            }
        };

        // Validate user token
        let jwt_token = auth.0.token;
        let request_user =
            match get_user_from_token(&mut tx, &mut redis_conn, jwt_token.clone()).await {
                Ok(val) => val,
                Err(err) => {
                    return CreateRolePermissionResponses::InternalServerError(Json(
                        InternalServerErrorResponse::new(
                            "route.role_permission",
                            "create_role_permission_api",
                            "get user from token",
                            &err.to_string(),
                        ),
                    ))
                }
            };
        if request_user.is_none() {
            return CreateRolePermissionResponses::Unauthorized(Json(
                UnauthorizedResponse::default(),
            ));
        }
        let request_user = request_user.unwrap();

        // Validate
        let role_id = match Uuid::parse_str(&json.role_id) {
            Ok(val) => val,
            Err(_) => {
                return CreateRolePermissionResponses::BadRequest(Json(BadRequestResponse {
                    message: format!("role with id {} not found", json.role_id),
                }));
            }
        };
        let role = match get_role_by_id(&mut tx, &role_id).await {
            Ok(val) => val,
            Err(err) => {
                return CreateRolePermissionResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.role_permission",
                        "create_role_permission_api",
                        "get_role_by_id",
                        &err.to_string(),
                    ),
                ))
            }
        };
        if role.is_none() {
            return CreateRolePermissionResponses::BadRequest(Json(BadRequestResponse {
                message: format!("role with id {} not found", json.role_id),
            }));
        }

        let permission_id = match Uuid::parse_str(&json.permission_id) {
            Ok(val) => val,
            Err(_) => {
                return CreateRolePermissionResponses::BadRequest(Json(BadRequestResponse {
                    message: format!("permission with id {} not found", json.permission_id),
                }));
            }
        };
        let permission = match get_permission_by_id(&mut tx, &permission_id).await {
            Ok(val) => val,
            Err(err) => {
                return CreateRolePermissionResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.role_permission",
                        "create_role_permission_api",
                        "get_permission_by_id",
                        &err.to_string(),
                    ),
                ))
            }
        };
        if permission.is_none() {
            return CreateRolePermissionResponses::BadRequest(Json(BadRequestResponse {
                message: format!("permission with id {} not found", json.permission_id),
            }));
        }

        let attribute_id = match Uuid::parse_str(&json.attribute_id) {
            Ok(val) => val,
            Err(_) => {
                return CreateRolePermissionResponses::BadRequest(Json(BadRequestResponse {
                    message: format!("attribute with id {} not found", json.attribute_id),
                }));
            }
        };
        let attribute = match get_permission_attribute_by_id(&mut tx, &attribute_id).await {
            Ok(val) => val,
            Err(err) => {
                return CreateRolePermissionResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.role_permission",
                        "create_role_permission_api",
                        "get_permission_attribute_by_id",
                        &err.to_string(),
                    ),
                ))
            }
        };
        if attribute.is_none() {
            return CreateRolePermissionResponses::BadRequest(Json(BadRequestResponse {
                message: format!("attribute with id {} not found", json.attribute_id),
            }));
        }
        let role_permission = match get_detail_role_permission(
            &mut tx,
            &role_id,
            &permission_id,
            &attribute_id,
        )
        .await
        {
            Ok(val) => val,
            Err(err) => {
                return CreateRolePermissionResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.role_permission",
                        "create_role_permission_api",
                        "get_detail_role_permission",
                        &err.to_string(),
                    ),
                ))
            }
        };
        if role_permission.is_some() {
            return CreateRolePermissionResponses::BadRequest(Json(BadRequestResponse { message: format!("role_permission with role_id = {}, permission_id = {}, attribute_id = {} already exists", json.role_id, json.permission_id, json.attribute_id)}));
        }
        let now = Local::now().fixed_offset();
        let new_role_permision = RolePermission {
            role_id,
            permission_id,
            attribute_id,
            created_by: Some(request_user.id),
            updated_by: Some(request_user.id),
            created_date: Some(now),
            updated_date: Some(now),
        };
        if let Err(err) = create_role_permission(&mut tx, &new_role_permision).await {
            return CreateRolePermissionResponses::InternalServerError(Json(
                InternalServerErrorResponse::new(
                    "route.role_permission",
                    "create_role_permission_api",
                    "create_role_permission",
                    &err.to_string(),
                ),
            ));
        }
        if let Err(err) = tx.commit().await {
            return CreateRolePermissionResponses::InternalServerError(Json(
                InternalServerErrorResponse::new(
                    "route.role_permission",
                    "create_role_permission_api",
                    "commit transaction",
                    &err.to_string(),
                ),
            ));
        }
        CreateRolePermissionResponses::Ok(Json(RolePermissionCreateResponse {
            role_id: new_role_permision.role_id.to_string(),
            permission_id: new_role_permision.permission_id.to_string(),
            attribute_id: new_role_permision.attribute_id.to_string(),
        }))
    }

    #[oai(
        path = "/role-permissions",
        method = "delete",
        tag = "ApiRolePermissionTags::RolePermission"
    )]
    async fn delete_role_permission_api(
        &self,
        Query(role_id): Query<String>,
        Query(permission_id): Query<String>,
        Query(attribute_id): Query<String>,
        state: Data<&Arc<AppState>>,
        auth: BearerAuthorization,
    ) -> DeleteRolePermissionResponses {
        // Begin db transaction
        let mut tx = match state.db.begin().await {
            Ok(val) => val,
            Err(err) => {
                return DeleteRolePermissionResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.role_permission",
                        "delete_role_permission_api",
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
                return DeleteRolePermissionResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.role_permission",
                        "delete_role_permission_api",
                        "get redis pool connection",
                        &err.to_string(),
                    ),
                ))
            }
        };

        // Validate user token
        let jwt_token = auth.0.token;
        let request_user =
            match get_user_from_token(&mut tx, &mut redis_conn, jwt_token.clone()).await {
                Ok(val) => val,
                Err(err) => {
                    return DeleteRolePermissionResponses::InternalServerError(Json(
                        InternalServerErrorResponse::new(
                            "route.role_permission",
                            "delete_role_permission_api",
                            "get user from token",
                            &err.to_string(),
                        ),
                    ))
                }
            };
        if request_user.is_none() {
            return DeleteRolePermissionResponses::Unauthorized(Json(
                UnauthorizedResponse::default(),
            ));
        }

        // Validate
        let role_id = match Uuid::parse_str(&role_id) {
            Ok(val) => val,
            Err(_) => {
                return DeleteRolePermissionResponses::BadRequest(Json(BadRequestResponse {
                    message: format!("role with id {} not found", role_id),
                }));
            }
        };
        let role = match get_role_by_id(&mut tx, &role_id).await {
            Ok(val) => val,
            Err(err) => {
                return DeleteRolePermissionResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.role_permission",
                        "delete_role_permission_api",
                        "get_role_by_id",
                        &err.to_string(),
                    ),
                ))
            }
        };
        if role.is_none() {
            return DeleteRolePermissionResponses::BadRequest(Json(BadRequestResponse {
                message: format!("role with id {} not found", role_id),
            }));
        }

        let permission_id = match Uuid::parse_str(&permission_id) {
            Ok(val) => val,
            Err(_) => {
                return DeleteRolePermissionResponses::BadRequest(Json(BadRequestResponse {
                    message: format!("permission with id {} not found", permission_id),
                }));
            }
        };
        let permission = match get_permission_by_id(&mut tx, &permission_id).await {
            Ok(val) => val,
            Err(err) => {
                return DeleteRolePermissionResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.role_permission",
                        "delete_role_permission_api",
                        "get_permission_by_id",
                        &err.to_string(),
                    ),
                ))
            }
        };
        if permission.is_none() {
            return DeleteRolePermissionResponses::BadRequest(Json(BadRequestResponse {
                message: format!("permission with id {} not found", permission_id),
            }));
        }

        let attribute_id = match Uuid::parse_str(&attribute_id) {
            Ok(val) => val,
            Err(_) => {
                return DeleteRolePermissionResponses::BadRequest(Json(BadRequestResponse {
                    message: format!("attribute with id {} not found", attribute_id),
                }));
            }
        };
        let attribute = match get_permission_attribute_by_id(&mut tx, &attribute_id).await {
            Ok(val) => val,
            Err(err) => {
                return DeleteRolePermissionResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.role_permission",
                        "delete_role_permission_api",
                        "get_permission_attribute_by_id",
                        &err.to_string(),
                    ),
                ))
            }
        };
        if attribute.is_none() {
            return DeleteRolePermissionResponses::BadRequest(Json(BadRequestResponse {
                message: format!("attribute with id {} not found", attribute_id),
            }));
        }
        let role_permission = match get_detail_role_permission(
            &mut tx,
            &role_id,
            &permission_id,
            &attribute_id,
        )
        .await
        {
            Ok(val) => val,
            Err(err) => {
                return DeleteRolePermissionResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.role_permission",
                        "delete_role_permission_api",
                        "get_detail_role_permission",
                        &err.to_string(),
                    ),
                ))
            }
        };
        if role_permission.is_none() {
            return DeleteRolePermissionResponses::NotFound(Json(NotFoundResponse{
                message: format!("role_permission with role_id = {}, permission_id = {}, attribute_id = {} not exists", role_id, permission_id, attribute_id)
            }));
        }
        if let Err(err) = delete_role_permission(&mut tx, &role_permission.unwrap()).await {
            return DeleteRolePermissionResponses::InternalServerError(Json(
                InternalServerErrorResponse::new(
                    "route.role_permission",
                    "delete_role_permission_api",
                    "delete_role_permission",
                    &err.to_string(),
                ),
            ));
        }
        if let Err(err) = tx.commit().await {
            return DeleteRolePermissionResponses::InternalServerError(Json(
                InternalServerErrorResponse::new(
                    "route.role_permission",
                    "delete_role_permission_api",
                    "commit transaction",
                    &err.to_string(),
                ),
            ));
        }
        DeleteRolePermissionResponses::NoContent
    }
}
