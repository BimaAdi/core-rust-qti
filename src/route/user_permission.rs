use std::sync::Arc;

use chrono::Local;
use poem::web::Data;
use poem_openapi::{param::Query, payload::Json, OpenApi, Tags};
use uuid::Uuid;

use crate::{
    core::security::{get_user_from_token, BearerAuthorization},
    model::user_permission::UserPermission,
    repository::{
        permission::get_permission_by_id,
        permission_attribute::get_permission_attribute_by_id,
        user::get_user_by_id,
        user_permission::{
            create_user_permission, delete_user_permission, get_all_user_permission,
            get_detail_user_permission,
        },
    },
    schema::{
        common::{
            BadRequestResponse, InternalServerErrorResponse, NotFoundResponse, PaginateResponse,
            UnauthorizedResponse,
        },
        user_permission::{
            CreateUserPermissionResponses, DeleteUserPermissionResponses,
            DetailPermissionAttributeUserPermission, DetailPermissionUserPermission,
            DetailUserPermissionResponse, DetailUserUserPermission,
            PaginateUserPermissionResponses, UserPermissionCreateRequest,
            UserPermissionCreateResponse,
        },
    },
    AppState,
};

#[derive(Tags)]
enum ApiUserPermissionTags {
    UserPermission,
}

pub struct ApiUserPermission;

#[OpenApi]
impl ApiUserPermission {
    #[oai(
        path = "/user-permissions",
        method = "get",
        tag = "ApiUserPermissionTags::UserPermission"
    )]
    async fn paginate_user_permission_api(
        &self,
        Query(user_id): Query<String>,
        Query(page): Query<Option<u32>>,
        Query(page_size): Query<Option<u32>>,
        Query(all): Query<Option<bool>>,
        state: Data<&Arc<AppState>>,
        auth: BearerAuthorization,
    ) -> PaginateUserPermissionResponses {
        // Begin db transaction
        let mut tx = match state.db.begin().await {
            Ok(val) => val,
            Err(err) => {
                return PaginateUserPermissionResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.user_permission",
                        "paginate_user_permission_api",
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
                return PaginateUserPermissionResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.user_permission",
                        "paginate_user_permission_api",
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
                    return PaginateUserPermissionResponses::InternalServerError(Json(
                        InternalServerErrorResponse::new(
                            "route.user_permission",
                            "paginate_user_permission_api",
                            "get user from token",
                            &err.to_string(),
                        ),
                    ))
                }
            };
        if request_user.is_none() {
            return PaginateUserPermissionResponses::Unauthorized(Json(
                UnauthorizedResponse::default(),
            ));
        }

        // Validasi
        let user_id = match Uuid::parse_str(&user_id) {
            Ok(val) => val,
            Err(_) => {
                return PaginateUserPermissionResponses::BadRequest(Json(BadRequestResponse {
                    message: format!("user with id = {} not found", user_id),
                }))
            }
        };
        let (user, _) = match get_user_by_id(&mut tx, &user_id, None).await {
            Ok(val) => val,
            Err(err) => {
                return PaginateUserPermissionResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.user_permission",
                        "paginate_user_permission_api",
                        "get_user_by_id",
                        &err.to_string(),
                    ),
                ))
            }
        };
        if user.is_none() {
            return PaginateUserPermissionResponses::BadRequest(Json(BadRequestResponse {
                message: format!("user with id = {} not found", user_id),
            }));
        }
        let user = user.unwrap();

        let page = page.unwrap_or(1);
        let page_size = page_size.unwrap_or(10);
        let (data, counts, page_count) = match get_all_user_permission(
            &mut tx,
            Some(page),
            Some(page_size),
            &user_id,
            all,
        )
        .await
        {
            Ok(val) => val,
            Err(err) => {
                return PaginateUserPermissionResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.user_permission",
                        "paginate_user_permission_api",
                        "get_all_user_permission",
                        &err.to_string(),
                    ),
                ))
            }
        };

        let mut results: Vec<DetailUserPermissionResponse> = vec![];
        for item in data {
            let permission = match get_permission_by_id(&mut tx, &item.permission_id).await {
                Ok(val) => val.unwrap(),
                Err(err) => {
                    return PaginateUserPermissionResponses::InternalServerError(Json(
                        InternalServerErrorResponse::new(
                            "route.user_permission",
                            "paginate_user_permission_api",
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
                    return PaginateUserPermissionResponses::InternalServerError(Json(
                        InternalServerErrorResponse::new(
                            "route.user_permission",
                            "paginate_user_permission_api",
                            "get_permission_attribute_by_id",
                            &err.to_string(),
                        ),
                    ))
                }
            };
            results.push(DetailUserPermissionResponse {
                user: DetailUserUserPermission {
                    id: user.id.to_string(),
                    user_name: user.user_name.clone(),
                },
                permission: DetailPermissionUserPermission {
                    id: permission.id.to_string(),
                    permission_name: permission.permission_name,
                },
                permission_attribute: DetailPermissionAttributeUserPermission {
                    id: attribute.id.to_string(),
                    name: attribute.name,
                },
            });
        }
        PaginateUserPermissionResponses::Ok(Json(PaginateResponse {
            counts,
            page,
            page_count,
            page_size,
            results,
        }))
    }

    #[oai(
        path = "/user-permissions",
        method = "post",
        tag = "ApiUserPermissionTags::UserPermission"
    )]
    async fn create_user_permission_api(
        &self,
        Json(json): Json<UserPermissionCreateRequest>,
        state: Data<&Arc<AppState>>,
        auth: BearerAuthorization,
    ) -> CreateUserPermissionResponses {
        // Begin db transaction
        let mut tx = match state.db.begin().await {
            Ok(val) => val,
            Err(err) => {
                return CreateUserPermissionResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.user_permission",
                        "create_user_permission_api",
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
                return CreateUserPermissionResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.user_permission",
                        "create_user_permission_api",
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
                    return CreateUserPermissionResponses::InternalServerError(Json(
                        InternalServerErrorResponse::new(
                            "route.user_permission",
                            "create_user_permission_api",
                            "get user from token",
                            &err.to_string(),
                        ),
                    ))
                }
            };
        if request_user.is_none() {
            return CreateUserPermissionResponses::Unauthorized(Json(
                UnauthorizedResponse::default(),
            ));
        }
        let request_user = request_user.unwrap();

        // Validate
        let user_id = match Uuid::parse_str(&json.user_id) {
            Ok(val) => val,
            Err(_) => {
                return CreateUserPermissionResponses::BadRequest(Json(BadRequestResponse {
                    message: format!("user with id {} not found", json.user_id),
                }));
            }
        };
        let (user, _) = match get_user_by_id(&mut tx, &user_id, None).await {
            Ok(val) => val,
            Err(err) => {
                return CreateUserPermissionResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.user_permission",
                        "create_user_permission_api",
                        "get_user_by_id",
                        &err.to_string(),
                    ),
                ))
            }
        };
        if user.is_none() {
            return CreateUserPermissionResponses::BadRequest(Json(BadRequestResponse {
                message: format!("user with id {} not found", json.user_id),
            }));
        }

        let permission_id = match Uuid::parse_str(&json.permission_id) {
            Ok(val) => val,
            Err(_) => {
                return CreateUserPermissionResponses::BadRequest(Json(BadRequestResponse {
                    message: format!("permission with id {} not found", json.permission_id),
                }));
            }
        };
        let permission = match get_permission_by_id(&mut tx, &permission_id).await {
            Ok(val) => val,
            Err(err) => {
                return CreateUserPermissionResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.user_permission",
                        "create_user_permission_api",
                        "get_permission_by_id",
                        &err.to_string(),
                    ),
                ))
            }
        };
        if permission.is_none() {
            return CreateUserPermissionResponses::BadRequest(Json(BadRequestResponse {
                message: format!("permission with id {} not found", json.permission_id),
            }));
        }

        let attribute_id = match Uuid::parse_str(&json.attribute_id) {
            Ok(val) => val,
            Err(_) => {
                return CreateUserPermissionResponses::BadRequest(Json(BadRequestResponse {
                    message: format!("attribute with id {} not found", json.attribute_id),
                }));
            }
        };
        let attribute = match get_permission_attribute_by_id(&mut tx, &attribute_id).await {
            Ok(val) => val,
            Err(err) => {
                return CreateUserPermissionResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.user_permission",
                        "create_user_permission_api",
                        "get_permission_attribute_by_id",
                        &err.to_string(),
                    ),
                ))
            }
        };
        if attribute.is_none() {
            return CreateUserPermissionResponses::BadRequest(Json(BadRequestResponse {
                message: format!("attribute with id {} not found", json.attribute_id),
            }));
        }
        let user_permission = match get_detail_user_permission(
            &mut tx,
            &user_id,
            &permission_id,
            &attribute_id,
        )
        .await
        {
            Ok(val) => val,
            Err(err) => {
                return CreateUserPermissionResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.user_permission",
                        "create_user_permission_api",
                        "get_detail_user_permission",
                        &err.to_string(),
                    ),
                ))
            }
        };
        if user_permission.is_some() {
            return CreateUserPermissionResponses::BadRequest(Json(BadRequestResponse { message: format!("user_permission with user_id = {}, permission_id = {}, attribute_id = {} already exists", json.user_id, json.permission_id, json.attribute_id)}));
        }
        let now = Local::now().fixed_offset();
        let new_user_permision = UserPermission {
            user_id,
            permission_id,
            attribute_id,
            created_by: Some(request_user.id),
            updated_by: Some(request_user.id),
            created_date: Some(now),
            updated_date: Some(now),
        };
        if let Err(err) = create_user_permission(&mut tx, &new_user_permision).await {
            return CreateUserPermissionResponses::InternalServerError(Json(
                InternalServerErrorResponse::new(
                    "route.user_permission",
                    "create_user_permission_api",
                    "create_user_permission",
                    &err.to_string(),
                ),
            ));
        }
        if let Err(err) = tx.commit().await {
            return CreateUserPermissionResponses::InternalServerError(Json(
                InternalServerErrorResponse::new(
                    "route.user_permission",
                    "create_user_permission_api",
                    "commit transaction",
                    &err.to_string(),
                ),
            ));
        }
        CreateUserPermissionResponses::Ok(Json(UserPermissionCreateResponse {
            user_id: new_user_permision.user_id.to_string(),
            permission_id: new_user_permision.permission_id.to_string(),
            attribute_id: new_user_permision.attribute_id.to_string(),
        }))
    }

    #[oai(
        path = "/user-permissions",
        method = "delete",
        tag = "ApiUserPermissionTags::UserPermission"
    )]
    async fn delete_user_permission_api(
        &self,
        Query(user_id): Query<String>,
        Query(permission_id): Query<String>,
        Query(attribute_id): Query<String>,
        state: Data<&Arc<AppState>>,
        auth: BearerAuthorization,
    ) -> DeleteUserPermissionResponses {
        // Begin db transaction
        let mut tx = match state.db.begin().await {
            Ok(val) => val,
            Err(err) => {
                return DeleteUserPermissionResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.user_permission",
                        "delete_user_permission_api",
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
                return DeleteUserPermissionResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.user_permission",
                        "delete_user_permission_api",
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
                    return DeleteUserPermissionResponses::InternalServerError(Json(
                        InternalServerErrorResponse::new(
                            "route.user_permission",
                            "delete_user_permission_api",
                            "get user from token",
                            &err.to_string(),
                        ),
                    ))
                }
            };
        if request_user.is_none() {
            return DeleteUserPermissionResponses::Unauthorized(Json(
                UnauthorizedResponse::default(),
            ));
        }

        // Validate
        let user_id = match Uuid::parse_str(&user_id) {
            Ok(val) => val,
            Err(_) => {
                return DeleteUserPermissionResponses::BadRequest(Json(BadRequestResponse {
                    message: format!("user with id {} not found", user_id),
                }));
            }
        };
        let (user, _) = match get_user_by_id(&mut tx, &user_id, None).await {
            Ok(val) => val,
            Err(err) => {
                return DeleteUserPermissionResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.user_permission",
                        "delete_user_permission_api",
                        "get_user_by_id",
                        &err.to_string(),
                    ),
                ))
            }
        };
        if user.is_none() {
            return DeleteUserPermissionResponses::BadRequest(Json(BadRequestResponse {
                message: format!("user with id {} not found", user_id),
            }));
        }

        let permission_id = match Uuid::parse_str(&permission_id) {
            Ok(val) => val,
            Err(_) => {
                return DeleteUserPermissionResponses::BadRequest(Json(BadRequestResponse {
                    message: format!("permission with id {} not found", permission_id),
                }));
            }
        };
        let permission = match get_permission_by_id(&mut tx, &permission_id).await {
            Ok(val) => val,
            Err(err) => {
                return DeleteUserPermissionResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.user_permission",
                        "delete_user_permission_api",
                        "get_permission_by_id",
                        &err.to_string(),
                    ),
                ))
            }
        };
        if permission.is_none() {
            return DeleteUserPermissionResponses::BadRequest(Json(BadRequestResponse {
                message: format!("permission with id {} not found", permission_id),
            }));
        }

        let attribute_id = match Uuid::parse_str(&attribute_id) {
            Ok(val) => val,
            Err(_) => {
                return DeleteUserPermissionResponses::BadRequest(Json(BadRequestResponse {
                    message: format!("attribute with id {} not found", attribute_id),
                }));
            }
        };
        let attribute = match get_permission_attribute_by_id(&mut tx, &attribute_id).await {
            Ok(val) => val,
            Err(err) => {
                return DeleteUserPermissionResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.user_permission",
                        "delete_user_permission_api",
                        "get_permission_attribute_by_id",
                        &err.to_string(),
                    ),
                ))
            }
        };
        if attribute.is_none() {
            return DeleteUserPermissionResponses::BadRequest(Json(BadRequestResponse {
                message: format!("attribute with id {} not found", attribute_id),
            }));
        }
        let user_permission = match get_detail_user_permission(
            &mut tx,
            &user_id,
            &permission_id,
            &attribute_id,
        )
        .await
        {
            Ok(val) => val,
            Err(err) => {
                return DeleteUserPermissionResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.user_permission",
                        "delete_user_permission_api",
                        "get_detail_user_permission",
                        &err.to_string(),
                    ),
                ))
            }
        };
        if user_permission.is_none() {
            return DeleteUserPermissionResponses::NotFound(Json(NotFoundResponse{
                message: format!("user_permission with user_id = {}, permission_id = {}, attribute_id = {} not exists", user_id, permission_id, attribute_id)
            }));
        }
        if let Err(err) = delete_user_permission(&mut tx, &user_permission.unwrap()).await {
            return DeleteUserPermissionResponses::InternalServerError(Json(
                InternalServerErrorResponse::new(
                    "route.user_permission",
                    "delete_user_permission_api",
                    "delete_user_permission",
                    &err.to_string(),
                ),
            ));
        }
        if let Err(err) = tx.commit().await {
            return DeleteUserPermissionResponses::InternalServerError(Json(
                InternalServerErrorResponse::new(
                    "route.user_permission",
                    "delete_user_permission_api",
                    "commit transaction",
                    &err.to_string(),
                ),
            ));
        }
        DeleteUserPermissionResponses::NoContent
    }
}
