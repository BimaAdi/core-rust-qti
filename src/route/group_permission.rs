use std::sync::Arc;

use chrono::Local;
use poem::web::Data;
use poem_openapi::{param::Query, payload::Json, OpenApi, Tags};
use uuid::Uuid;

use crate::{
    core::security::{get_user_from_token, BearerAuthorization},
    model::group_permission::GroupPermission,
    repository::{
        group::get_group_by_id,
        group_permission::{
            create_group_permission, delete_group_permission, get_all_group_permission,
            get_detail_group_permission,
        },
        permission::get_permission_by_id,
        permission_attribute::get_permission_attribute_by_id,
    },
    schema::{
        common::{
            BadRequestResponse, InternalServerErrorResponse, NotFoundResponse, PaginateResponse,
            UnauthorizedResponse,
        },
        group_permission::{
            CreateGroupPermissionResponses, DeleteGroupPermissionResponses,
            DetailGroupGroupPermission, DetailGroupPermission,
            DetailPermissionAttributeGroupPermission, DetailPermissionGroupPermission,
            GroupPermissionCreateRequest, GroupPermissionCreateResponse,
            PaginateGroupPermissionResponses,
        },
    },
    AppState,
};

#[derive(Tags)]
enum ApiGroupPermissionTags {
    GroupPermission,
}

pub struct ApiGroupPermission;

#[OpenApi]
impl ApiGroupPermission {
    #[oai(
        path = "/group-permissions",
        method = "get",
        tag = "ApiGroupPermissionTags::GroupPermission"
    )]
    async fn paginate_group_permission_api(
        &self,
        Query(group_id): Query<String>,
        Query(page): Query<Option<u32>>,
        Query(page_size): Query<Option<u32>>,
        Query(all): Query<Option<bool>>,
        state: Data<&Arc<AppState>>,
        auth: BearerAuthorization,
    ) -> PaginateGroupPermissionResponses {
        // Begin db transaction
        let mut tx = match state.db.begin().await {
            Ok(val) => val,
            Err(err) => {
                return PaginateGroupPermissionResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.group_permission",
                        "paginate_group_permission_api",
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
                return PaginateGroupPermissionResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.group_permission",
                        "paginate_group_permission_api",
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
                    return PaginateGroupPermissionResponses::InternalServerError(Json(
                        InternalServerErrorResponse::new(
                            "route.group_permission",
                            "paginate_group_permission_api",
                            "get user from token",
                            &err.to_string(),
                        ),
                    ))
                }
            };
        if request_user.is_none() {
            return PaginateGroupPermissionResponses::Unauthorized(Json(
                UnauthorizedResponse::default(),
            ));
        }

        // Validasi
        let group_id = match Uuid::parse_str(&group_id) {
            Ok(val) => val,
            Err(_) => {
                return PaginateGroupPermissionResponses::BadRequest(Json(BadRequestResponse {
                    message: format!("group with id = {} not found", group_id),
                }))
            }
        };
        let group = match get_group_by_id(&mut tx, &group_id).await {
            Ok(val) => val,
            Err(err) => {
                return PaginateGroupPermissionResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.group_permission",
                        "paginate_group_permission_api",
                        "get_group_by_id",
                        &err.to_string(),
                    ),
                ))
            }
        };
        if group.is_none() {
            return PaginateGroupPermissionResponses::BadRequest(Json(BadRequestResponse {
                message: format!("group with id = {} not found", group_id),
            }));
        }
        let group = group.unwrap();

        let page = page.unwrap_or(1);
        let page_size = page_size.unwrap_or(10);
        let (data, counts, page_count) =
            match get_all_group_permission(&mut tx, Some(page), Some(page_size), &group_id, all)
                .await
            {
                Ok(val) => val,
                Err(err) => {
                    return PaginateGroupPermissionResponses::InternalServerError(Json(
                        InternalServerErrorResponse::new(
                            "route.group_permission",
                            "paginate_group_permission_api",
                            "get_all_group_permission",
                            &err.to_string(),
                        ),
                    ))
                }
            };

        let mut results: Vec<DetailGroupPermission> = vec![];
        for item in data {
            let permission = match get_permission_by_id(&mut tx, &item.permission_id).await {
                Ok(val) => val.unwrap(),
                Err(err) => {
                    return PaginateGroupPermissionResponses::InternalServerError(Json(
                        InternalServerErrorResponse::new(
                            "route.group_permission",
                            "paginate_group_permission_api",
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
                    return PaginateGroupPermissionResponses::InternalServerError(Json(
                        InternalServerErrorResponse::new(
                            "route.group_permission",
                            "paginate_group_permission_api",
                            "get_permission_attribute_by_id",
                            &err.to_string(),
                        ),
                    ))
                }
            };
            results.push(DetailGroupPermission {
                group: DetailGroupGroupPermission {
                    id: group.id.to_string(),
                    group_name: group.group_name.clone(),
                },
                permission: DetailPermissionGroupPermission {
                    id: permission.id.to_string(),
                    permission_name: permission.permission_name,
                },
                permission_attribute: DetailPermissionAttributeGroupPermission {
                    id: attribute.id.to_string(),
                    name: attribute.name,
                },
            });
        }
        PaginateGroupPermissionResponses::Ok(Json(PaginateResponse {
            counts,
            page,
            page_count,
            page_size,
            results,
        }))
    }

    #[oai(
        path = "/group-permissions",
        method = "post",
        tag = "ApiGroupPermissionTags::GroupPermission"
    )]
    async fn create_group_permission_api(
        &self,
        Json(json): Json<GroupPermissionCreateRequest>,
        state: Data<&Arc<AppState>>,
        auth: BearerAuthorization,
    ) -> CreateGroupPermissionResponses {
        // Begin db transaction
        let mut tx = match state.db.begin().await {
            Ok(val) => val,
            Err(err) => {
                return CreateGroupPermissionResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.group_permission",
                        "create_group_permission_api",
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
                return CreateGroupPermissionResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.group_permission",
                        "create_group_permission_api",
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
                    return CreateGroupPermissionResponses::InternalServerError(Json(
                        InternalServerErrorResponse::new(
                            "route.group_permission",
                            "create_group_permission_api",
                            "get user from token",
                            &err.to_string(),
                        ),
                    ))
                }
            };
        if request_user.is_none() {
            return CreateGroupPermissionResponses::Unauthorized(Json(
                UnauthorizedResponse::default(),
            ));
        }
        let request_user = request_user.unwrap();

        // Validate
        let group_id = match Uuid::parse_str(&json.group_id) {
            Ok(val) => val,
            Err(_) => {
                return CreateGroupPermissionResponses::BadRequest(Json(BadRequestResponse {
                    message: format!("group with id {} not found", json.group_id),
                }));
            }
        };
        let group = match get_group_by_id(&mut tx, &group_id).await {
            Ok(val) => val,
            Err(err) => {
                return CreateGroupPermissionResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.group_permission",
                        "create_group_permission_api",
                        "get_group_by_id",
                        &err.to_string(),
                    ),
                ))
            }
        };
        if group.is_none() {
            return CreateGroupPermissionResponses::BadRequest(Json(BadRequestResponse {
                message: format!("group with id {} not found", json.group_id),
            }));
        }

        let permission_id = match Uuid::parse_str(&json.permission_id) {
            Ok(val) => val,
            Err(_) => {
                return CreateGroupPermissionResponses::BadRequest(Json(BadRequestResponse {
                    message: format!("permission with id {} not found", json.permission_id),
                }));
            }
        };
        let permission = match get_permission_by_id(&mut tx, &permission_id).await {
            Ok(val) => val,
            Err(err) => {
                return CreateGroupPermissionResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.group_permission",
                        "create_group_permission_api",
                        "get_permission_by_id",
                        &err.to_string(),
                    ),
                ))
            }
        };
        if permission.is_none() {
            return CreateGroupPermissionResponses::BadRequest(Json(BadRequestResponse {
                message: format!("permission with id {} not found", json.permission_id),
            }));
        }

        let attribute_id = match Uuid::parse_str(&json.attribute_id) {
            Ok(val) => val,
            Err(_) => {
                return CreateGroupPermissionResponses::BadRequest(Json(BadRequestResponse {
                    message: format!("attribute with id {} not found", json.attribute_id),
                }));
            }
        };
        let attribute = match get_permission_attribute_by_id(&mut tx, &attribute_id).await {
            Ok(val) => val,
            Err(err) => {
                return CreateGroupPermissionResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.group_permission",
                        "create_group_permission_api",
                        "get_permission_attribute_by_id",
                        &err.to_string(),
                    ),
                ))
            }
        };
        if attribute.is_none() {
            return CreateGroupPermissionResponses::BadRequest(Json(BadRequestResponse {
                message: format!("attribute with id {} not found", json.attribute_id),
            }));
        }
        let group_permission =
            match get_detail_group_permission(&mut tx, &group_id, &permission_id, &attribute_id)
                .await
            {
                Ok(val) => val,
                Err(err) => {
                    return CreateGroupPermissionResponses::InternalServerError(Json(
                        InternalServerErrorResponse::new(
                            "route.group_permission",
                            "create_group_permission_api",
                            "get_detail_group_permission",
                            &err.to_string(),
                        ),
                    ))
                }
            };
        if group_permission.is_some() {
            return CreateGroupPermissionResponses::BadRequest(Json(BadRequestResponse { message: format!("group_permission with group_id = {}, permission_id = {}, attribute_id = {} already exists", json.group_id, json.permission_id, json.attribute_id)}));
        }
        let now = Local::now().fixed_offset();
        let new_group_permision = GroupPermission {
            group_id,
            permission_id,
            attribute_id,
            created_by: Some(request_user.id),
            updated_by: Some(request_user.id),
            created_date: Some(now),
            updated_date: Some(now),
        };
        if let Err(err) = create_group_permission(&mut tx, &new_group_permision).await {
            return CreateGroupPermissionResponses::InternalServerError(Json(
                InternalServerErrorResponse::new(
                    "route.group_permission",
                    "create_group_permission_api",
                    "create_group_permission",
                    &err.to_string(),
                ),
            ));
        }
        if let Err(err) = tx.commit().await {
            return CreateGroupPermissionResponses::InternalServerError(Json(
                InternalServerErrorResponse::new(
                    "route.group_permission",
                    "create_group_permission_api",
                    "commit transaction",
                    &err.to_string(),
                ),
            ));
        }
        CreateGroupPermissionResponses::Ok(Json(GroupPermissionCreateResponse {
            group_id: new_group_permision.group_id.to_string(),
            permission_id: new_group_permision.permission_id.to_string(),
            attribute_id: new_group_permision.attribute_id.to_string(),
        }))
    }

    #[oai(
        path = "/group-permissions",
        method = "delete",
        tag = "ApiGroupPermissionTags::GroupPermission"
    )]
    async fn delete_group_permission_api(
        &self,
        Query(group_id): Query<String>,
        Query(permission_id): Query<String>,
        Query(attribute_id): Query<String>,
        state: Data<&Arc<AppState>>,
        auth: BearerAuthorization,
    ) -> DeleteGroupPermissionResponses {
        // Begin db transaction
        let mut tx = match state.db.begin().await {
            Ok(val) => val,
            Err(err) => {
                return DeleteGroupPermissionResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.group_permission",
                        "delete_group_permission_api",
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
                return DeleteGroupPermissionResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.group_permission",
                        "delete_group_permission_api",
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
                    return DeleteGroupPermissionResponses::InternalServerError(Json(
                        InternalServerErrorResponse::new(
                            "route.group_permission",
                            "delete_group_permission_api",
                            "get user from token",
                            &err.to_string(),
                        ),
                    ))
                }
            };
        if request_user.is_none() {
            return DeleteGroupPermissionResponses::Unauthorized(Json(
                UnauthorizedResponse::default(),
            ));
        }

        // Validate
        let group_id = match Uuid::parse_str(&group_id) {
            Ok(val) => val,
            Err(_) => {
                return DeleteGroupPermissionResponses::BadRequest(Json(BadRequestResponse {
                    message: format!("group with id {} not found", group_id),
                }));
            }
        };
        let group = match get_group_by_id(&mut tx, &group_id).await {
            Ok(val) => val,
            Err(err) => {
                return DeleteGroupPermissionResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.group_permission",
                        "delete_group_permission_api",
                        "get_group_by_id",
                        &err.to_string(),
                    ),
                ))
            }
        };
        if group.is_none() {
            return DeleteGroupPermissionResponses::BadRequest(Json(BadRequestResponse {
                message: format!("group with id {} not found", group_id),
            }));
        }

        let permission_id = match Uuid::parse_str(&permission_id) {
            Ok(val) => val,
            Err(_) => {
                return DeleteGroupPermissionResponses::BadRequest(Json(BadRequestResponse {
                    message: format!("permission with id {} not found", permission_id),
                }));
            }
        };
        let permission = match get_permission_by_id(&mut tx, &permission_id).await {
            Ok(val) => val,
            Err(err) => {
                return DeleteGroupPermissionResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.group_permission",
                        "delete_group_permission_api",
                        "get_permission_by_id",
                        &err.to_string(),
                    ),
                ))
            }
        };
        if permission.is_none() {
            return DeleteGroupPermissionResponses::BadRequest(Json(BadRequestResponse {
                message: format!("permission with id {} not found", permission_id),
            }));
        }

        let attribute_id = match Uuid::parse_str(&attribute_id) {
            Ok(val) => val,
            Err(_) => {
                return DeleteGroupPermissionResponses::BadRequest(Json(BadRequestResponse {
                    message: format!("attribute with id {} not found", attribute_id),
                }));
            }
        };
        let attribute = match get_permission_attribute_by_id(&mut tx, &attribute_id).await {
            Ok(val) => val,
            Err(err) => {
                return DeleteGroupPermissionResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.group_permission",
                        "delete_group_permission_api",
                        "get_permission_attribute_by_id",
                        &err.to_string(),
                    ),
                ))
            }
        };
        if attribute.is_none() {
            return DeleteGroupPermissionResponses::BadRequest(Json(BadRequestResponse {
                message: format!("attribute with id {} not found", attribute_id),
            }));
        }
        let group_permission =
            match get_detail_group_permission(&mut tx, &group_id, &permission_id, &attribute_id)
                .await
            {
                Ok(val) => val,
                Err(err) => {
                    return DeleteGroupPermissionResponses::InternalServerError(Json(
                        InternalServerErrorResponse::new(
                            "route.group_permission",
                            "delete_group_permission_api",
                            "get_detail_group_permission",
                            &err.to_string(),
                        ),
                    ))
                }
            };
        if group_permission.is_none() {
            return DeleteGroupPermissionResponses::NotFound(Json(NotFoundResponse{
                message: format!("group_permission with group_id = {}, permission_id = {}, attribute_id = {} not exists", group_id, permission_id, attribute_id)
            }));
        }
        if let Err(err) = delete_group_permission(&mut tx, &group_permission.unwrap()).await {
            return DeleteGroupPermissionResponses::InternalServerError(Json(
                InternalServerErrorResponse::new(
                    "route.group_permission",
                    "delete_group_permission_api",
                    "delete_group_permission",
                    &err.to_string(),
                ),
            ));
        }
        if let Err(err) = tx.commit().await {
            return DeleteGroupPermissionResponses::InternalServerError(Json(
                InternalServerErrorResponse::new(
                    "route.group_permission",
                    "delete_group_permission_api",
                    "commit transaction",
                    &err.to_string(),
                ),
            ));
        }
        DeleteGroupPermissionResponses::NoContent
    }
}
