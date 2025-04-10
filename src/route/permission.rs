use std::sync::Arc;

use chrono::Local;
use poem::web::Data;
use poem_openapi::{param::Query, payload::Json, OpenApi, Tags};
use uuid::Uuid;

use crate::{
    core::{
        security::{get_user_from_token, BearerAuthorization},
        utils::datetime_to_string_opt,
    },
    model::{
        permission::Permission, permission_attribute::PermissionAttribute,
        permission_attribute_list::PermissionAttributeList, user::User,
    },
    repository::{
        permission::{
            create_permission, delete_permission, get_all_permission, get_permission_by_id,
            update_permission,
        },
        permission_attribute::{get_permission_attribute_by_id, get_permission_attribute_by_ids},
        permission_attribute_list::{
            create_permission_attribute_list, get_all_permission_attribute_list,
            update_permssion_attribute_list_by_permission,
        },
        user::get_user_by_id,
    },
    schema::{
        common::{
            BadRequestResponse, InternalServerErrorResponse, NotFoundResponse, PaginateResponse,
            UnauthorizedResponse,
        },
        permission::{
            AllPermissionResponses, DetailPermission, DetailUserPermission,
            DropdownPermissionResponses, PaginatePermissionResponses, PermissionAllResponse,
            PermissionAttributeListPermissionDetail, PermissionCreateRequest,
            PermissionCreateResponse, PermissionCreateResponses, PermissionDeleteResponses,
            PermissionDetailResponse, PermissionDetailResponses, PermissionDropdownResponse,
            PermissionUpdateRequest, PermissionUpdateResponse, PermissionUpdateResponses,
        },
    },
    AppState,
};

#[derive(Tags)]
enum ApiPermissionTags {
    Permission,
}

pub struct ApiPermission;

#[OpenApi]
impl ApiPermission {
    #[allow(clippy::too_many_arguments)]
    #[oai(
        path = "/permissions/",
        method = "get",
        tag = "ApiPermissionTags::Permission"
    )]
    async fn paginate_permission_api(
        &self,
        Query(page): Query<Option<u32>>,
        Query(page_size): Query<Option<u32>>,
        Query(search): Query<Option<String>>,
        Query(is_user): Query<Option<bool>>,
        Query(is_role): Query<Option<bool>>,
        Query(is_group): Query<Option<bool>>,
        state: Data<&Arc<AppState>>,
        auth: BearerAuthorization,
    ) -> PaginatePermissionResponses {
        // Begin db transaction
        let mut tx = match state.db.begin().await {
            Ok(val) => val,
            Err(err) => {
                return PaginatePermissionResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.permission",
                        "paginate_permission_api",
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
                return PaginatePermissionResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.permission",
                        "paginate_permission_api",
                        "get redis pool connection",
                        &err.to_string(),
                    ),
                ))
            }
        };

        // Validate user token
        let jwt_token = auth.0.token;
        let user = match get_user_from_token(&mut tx, &mut redis_conn, jwt_token.clone()).await {
            Ok(val) => val,
            Err(err) => {
                return PaginatePermissionResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.permission",
                        "paginate_permission_api",
                        "get user from token",
                        &err.to_string(),
                    ),
                ))
            }
        };
        if user.is_none() {
            return PaginatePermissionResponses::Unauthorized(
                Json(UnauthorizedResponse::default()),
            );
        }
        let (data, counts, page_count) = match get_all_permission(
            &mut tx, page, page_size, search, is_user, is_role, is_group, None, None,
        )
        .await
        {
            Ok(val) => val,
            Err(err) => {
                return PaginatePermissionResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.permission",
                        "paginate_permission_api",
                        "get_all_permission",
                        &err.to_string(),
                    ),
                ))
            }
        };
        let mut results: Vec<DetailPermission> = vec![];
        for item in data {
            let mut created_by: Option<User> = None;
            if item.created_by.is_some() {
                (created_by, _) =
                    match get_user_by_id(&mut tx, &item.created_by.unwrap(), Some(true)).await {
                        Ok(val) => val,
                        Err(err) => {
                            return PaginatePermissionResponses::InternalServerError(Json(
                                InternalServerErrorResponse::new(
                                    "route.permission",
                                    "paginate_permission_api",
                                    "get user created_by",
                                    &err.to_string(),
                                ),
                            ))
                        }
                    };
            }
            let mut updated_by: Option<User> = None;
            if item.updated_by.is_some() {
                (updated_by, _) =
                    match get_user_by_id(&mut tx, &item.updated_by.unwrap(), Some(true)).await {
                        Ok(val) => val,
                        Err(err) => {
                            return PaginatePermissionResponses::InternalServerError(Json(
                                InternalServerErrorResponse::new(
                                    "route.permission",
                                    "paginate_permission_api",
                                    "get user updated_by",
                                    &err.to_string(),
                                ),
                            ))
                        }
                    };
            }
            results.push(DetailPermission {
                id: item.id.to_string(),
                permission_name: item.permission_name,
                description: item.description,
                is_user: item.is_user.unwrap_or(false),
                is_role: item.is_role.unwrap_or(false),
                is_group: item.is_group.unwrap_or(false),
                created_date: datetime_to_string_opt(item.created_date),
                updated_date: datetime_to_string_opt(item.updated_date),
                created_by: created_by.map(|x| DetailUserPermission {
                    id: x.id.to_string(),
                    user_name: x.user_name,
                }),
                updated_by: updated_by.map(|x| DetailUserPermission {
                    id: x.id.to_string(),
                    user_name: x.user_name,
                }),
            });
        }
        PaginatePermissionResponses::Ok(Json(PaginateResponse {
            counts,
            page: page.unwrap_or(1),
            page_count,
            page_size: page_size.unwrap_or(10),
            results,
        }))
    }

    #[oai(
        path = "/permissions/all/",
        method = "get",
        tag = "ApiPermissionTags::Permission"
    )]
    async fn get_all_permission_api(
        &self,
        state: Data<&Arc<AppState>>,
        auth: BearerAuthorization,
    ) -> AllPermissionResponses {
        // Begin db transaction
        let mut tx = match state.db.begin().await {
            Ok(val) => val,
            Err(err) => {
                return AllPermissionResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.permission",
                        "get_all_permission_api",
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
                return AllPermissionResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.permission",
                        "get_all_permission_api",
                        "get redis pool connection",
                        &err.to_string(),
                    ),
                ))
            }
        };

        // Validate user token
        let jwt_token = auth.0.token;
        let user = match get_user_from_token(&mut tx, &mut redis_conn, jwt_token.clone()).await {
            Ok(val) => val,
            Err(err) => {
                return AllPermissionResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.permission",
                        "get_all_permission_api",
                        "get user from token",
                        &err.to_string(),
                    ),
                ))
            }
        };
        if user.is_none() {
            return AllPermissionResponses::Unauthorized(Json(UnauthorizedResponse::default()));
        }
        let (data, _, _) = match get_all_permission(
            &mut tx,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(true),
        )
        .await
        {
            Ok(val) => val,
            Err(err) => {
                return AllPermissionResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.permission",
                        "get_all_permission_api",
                        "get_all_permission",
                        &err.to_string(),
                    ),
                ))
            }
        };
        AllPermissionResponses::Ok(Json(
            data.iter()
                .map(|x| PermissionAllResponse {
                    id: x.id.to_string(),
                    permission_name: x.permission_name.clone(),
                    description: x.description.clone(),
                    is_user: x.is_user.unwrap_or(false),
                    is_role: x.is_role.unwrap_or(false),
                    is_group: x.is_group.unwrap_or(false),
                    created_date: datetime_to_string_opt(x.created_date),
                    updated_date: datetime_to_string_opt(x.updated_date),
                })
                .collect(),
        ))
    }

    #[allow(clippy::too_many_arguments)]
    #[oai(
        path = "/permissions/dropdown/",
        method = "get",
        tag = "ApiPermissionTags::Permission"
    )]
    async fn get_dropdown_permission_api(
        &self,
        state: Data<&Arc<AppState>>,
        auth: BearerAuthorization,
        Query(search): Query<Option<String>>,
        Query(is_user): Query<Option<bool>>,
        Query(is_role): Query<Option<bool>>,
        Query(is_group): Query<Option<bool>>,
        Query(limit): Query<Option<u32>>,
    ) -> DropdownPermissionResponses {
        // Begin db transaction
        let mut tx = match state.db.begin().await {
            Ok(val) => val,
            Err(err) => {
                return DropdownPermissionResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.permission",
                        "get_all_permission_api",
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
                return DropdownPermissionResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.permission",
                        "get_all_permission_api",
                        "get redis pool connection",
                        &err.to_string(),
                    ),
                ))
            }
        };

        // Validate user token
        let jwt_token = auth.0.token;
        let user = match get_user_from_token(&mut tx, &mut redis_conn, jwt_token.clone()).await {
            Ok(val) => val,
            Err(err) => {
                return DropdownPermissionResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.permission",
                        "get_all_permission_api",
                        "get user from token",
                        &err.to_string(),
                    ),
                ))
            }
        };
        if user.is_none() {
            return DropdownPermissionResponses::Unauthorized(
                Json(UnauthorizedResponse::default()),
            );
        }
        let (data, _, _) = match get_all_permission(
            &mut tx,
            None,
            None,
            search,
            is_user,
            is_role,
            is_group,
            limit,
            Some(true),
        )
        .await
        {
            Ok(val) => val,
            Err(err) => {
                return DropdownPermissionResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.permission",
                        "get_all_permission_api",
                        "get_all_permission",
                        &err.to_string(),
                    ),
                ))
            }
        };
        DropdownPermissionResponses::Ok(Json(
            data.iter()
                .map(|x| PermissionDropdownResponse {
                    id: x.id.to_string(),
                    permission_name: x.permission_name.clone(),
                })
                .collect(),
        ))
    }

    #[oai(
        path = "/permissions/detail/",
        method = "get",
        tag = "ApiPermissionTags::Permission"
    )]
    async fn get_detail_permission_api(
        &self,
        Query(id): Query<String>,
        state: Data<&Arc<AppState>>,
        auth: BearerAuthorization,
    ) -> PermissionDetailResponses {
        // Begin db transaction
        let mut tx = match state.db.begin().await {
            Ok(val) => val,
            Err(err) => {
                return PermissionDetailResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.permission",
                        "get_detail_permission_api",
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
                return PermissionDetailResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.permission",
                        "get_detail_permission_api",
                        "get redis pool connection",
                        &err.to_string(),
                    ),
                ))
            }
        };

        // Validate user token
        let jwt_token = auth.0.token;
        let user = match get_user_from_token(&mut tx, &mut redis_conn, jwt_token.clone()).await {
            Ok(val) => val,
            Err(err) => {
                return PermissionDetailResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.permission",
                        "get_detail_permission_api",
                        "get user from token",
                        &err.to_string(),
                    ),
                ))
            }
        };
        if user.is_none() {
            return PermissionDetailResponses::Unauthorized(Json(UnauthorizedResponse::default()));
        }

        // get detail permission
        let id = match Uuid::parse_str(&id) {
            Ok(val) => val,
            Err(_) => {
                return PermissionDetailResponses::NotFound(Json(NotFoundResponse {
                    message: format!("permission with id = {} not found", id),
                }))
            }
        };

        let data = match get_permission_by_id(&mut tx, &id).await {
            Ok(val) => val,
            Err(err) => {
                return PermissionDetailResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.permission",
                        "get_detail_permission_api",
                        "get_permission_by_id",
                        &err.to_string(),
                    ),
                ))
            }
        };
        if data.is_none() {
            return PermissionDetailResponses::NotFound(Json(NotFoundResponse {
                message: format!("permission with id = {} not found", id),
            }));
        }
        let data = data.unwrap();
        let mut created_by: Option<User> = None;
        if data.created_by.is_some() {
            (created_by, _) = match get_user_by_id(&mut tx, &data.id, Some(true)).await {
                Ok(val) => val,
                Err(err) => {
                    return PermissionDetailResponses::InternalServerError(Json(
                        InternalServerErrorResponse::new(
                            "route.permission",
                            "get_detail_permission_api",
                            "get user created_by",
                            &err.to_string(),
                        ),
                    ))
                }
            };
        }
        let mut updated_by: Option<User> = None;
        if data.updated_by.is_some() {
            (updated_by, _) = match get_user_by_id(&mut tx, &data.id, Some(true)).await {
                Ok(val) => val,
                Err(err) => {
                    return PermissionDetailResponses::InternalServerError(Json(
                        InternalServerErrorResponse::new(
                            "route.permission",
                            "get_detail_permission_api",
                            "get user updated_by",
                            &err.to_string(),
                        ),
                    ))
                }
            };
        }
        let permission_attribute_lists =
            match get_all_permission_attribute_list(&mut tx, Some(&data.id), None).await {
                Ok(val) => val,
                Err(err) => {
                    return PermissionDetailResponses::InternalServerError(Json(
                        InternalServerErrorResponse::new(
                            "route.permission",
                            "get_detail_permission_api",
                            "get_all_permission_attribute_list",
                            &err.to_string(),
                        ),
                    ))
                }
            };
        let attribute_ids: Vec<Uuid> = permission_attribute_lists
            .iter()
            .map(|x| x.attribute_id)
            .collect();
        let mut permission_attributes: Vec<PermissionAttribute> = vec![];
        if !attribute_ids.is_empty() {
            permission_attributes =
                match get_permission_attribute_by_ids(&mut tx, attribute_ids).await {
                    Ok(val) => val,
                    Err(err) => {
                        return PermissionDetailResponses::InternalServerError(Json(
                            InternalServerErrorResponse::new(
                                "route.permission",
                                "get_detail_permission_api",
                                "get_permission_attribute_by_ids",
                                &err.to_string(),
                            ),
                        ))
                    }
                };
        }
        PermissionDetailResponses::Ok(Json(PermissionDetailResponse {
            id: data.id.to_string(),
            permission_name: data.permission_name,
            description: data.description,
            is_user: data.is_user.unwrap_or(false),
            is_role: data.is_role.unwrap_or(false),
            is_group: data.is_group.unwrap_or(false),
            created_date: datetime_to_string_opt(data.created_date),
            updated_date: datetime_to_string_opt(data.updated_date),
            created_by: created_by.map(|x| DetailUserPermission {
                id: x.id.to_string(),
                user_name: x.user_name,
            }),
            updated_by: updated_by.map(|x| DetailUserPermission {
                id: x.id.to_string(),
                user_name: x.user_name,
            }),
            permission_attribute_ids: permission_attributes
                .iter()
                .map(|x| PermissionAttributeListPermissionDetail {
                    id: x.id.to_string(),
                    name: x.name.clone(),
                    description: x.description.clone(),
                })
                .collect(),
        }))
    }

    #[oai(
        path = "/permissions/",
        method = "post",
        tag = "ApiPermissionTags::Permission"
    )]
    async fn create_permission_api(
        &self,
        Json(json): Json<PermissionCreateRequest>,
        state: Data<&Arc<AppState>>,
        auth: BearerAuthorization,
    ) -> PermissionCreateResponses {
        // Begin db transaction
        let mut tx = match state.db.begin().await {
            Ok(val) => val,
            Err(err) => {
                return PermissionCreateResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.permission",
                        "create_permission_api",
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
                return PermissionCreateResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.permission",
                        "create_permission_api",
                        "get redis pool connection",
                        &err.to_string(),
                    ),
                ))
            }
        };

        // Validate user token
        let jwt_token = auth.0.token;
        let user = match get_user_from_token(&mut tx, &mut redis_conn, jwt_token.clone()).await {
            Ok(val) => val,
            Err(err) => {
                return PermissionCreateResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.permission",
                        "create_permission_api",
                        "get user from token",
                        &err.to_string(),
                    ),
                ))
            }
        };
        if user.is_none() {
            return PermissionCreateResponses::Unauthorized(Json(UnauthorizedResponse::default()));
        }
        // Validate json request
        let mut permission_attributes: Vec<PermissionAttribute> = vec![];
        for item in json.permission_attribute_ids {
            let permission_attribute_id = match Uuid::parse_str(&item) {
                Ok(val) => val,
                Err(_) => {
                    return PermissionCreateResponses::BadRequest(Json(BadRequestResponse {
                        message: format!("permission attribute id = {} not found", item),
                    }));
                }
            };
            let permission_attribute =
                match get_permission_attribute_by_id(&mut tx, &permission_attribute_id).await {
                    Ok(val) => val,
                    Err(err) => {
                        return PermissionCreateResponses::InternalServerError(Json(
                            InternalServerErrorResponse::new(
                                "route.permission",
                                "create_permission_api",
                                "get_permission_attribute_by_id",
                                &err.to_string(),
                            ),
                        ))
                    }
                };
            if permission_attribute.is_none() {
                return PermissionCreateResponses::BadRequest(Json(BadRequestResponse {
                    message: format!("permission attribute id = {} not found", item),
                }));
            }
            permission_attributes.push(permission_attribute.unwrap());
        }
        // Create permission
        let request_user = user.unwrap();
        let now = Local::now().fixed_offset();
        let new_permission = Permission {
            id: Uuid::now_v7(),
            permission_name: json.permission_name,
            is_user: Some(json.is_user),
            is_role: Some(json.is_role),
            is_group: Some(json.is_group),
            description: json.description,
            created_by: Some(request_user.id),
            updated_by: Some(request_user.id),
            created_date: Some(now),
            updated_date: Some(now),
        };
        if let Err(err) = create_permission(&mut tx, &new_permission).await {
            return PermissionCreateResponses::InternalServerError(Json(
                InternalServerErrorResponse::new(
                    "route.permission",
                    "create_permission_api",
                    "create_permission",
                    &err.to_string(),
                ),
            ));
        }
        for item in permission_attributes {
            let new_permission_attribute_list = PermissionAttributeList {
                permission_id: new_permission.id,
                attribute_id: item.id,
            };
            if let Err(err) =
                create_permission_attribute_list(&mut tx, &new_permission_attribute_list).await
            {
                return PermissionCreateResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.permission",
                        "create_permission_api",
                        "create_permission_attribute_list",
                        &err.to_string(),
                    ),
                ));
            }
        }
        if let Err(err) = tx.commit().await {
            return PermissionCreateResponses::InternalServerError(Json(
                InternalServerErrorResponse::new(
                    "route.permission",
                    "create_permission_api",
                    "commit transaction",
                    &err.to_string(),
                ),
            ));
        }
        PermissionCreateResponses::Created(Json(PermissionCreateResponse {
            id: new_permission.id.to_string(),
            permission_name: new_permission.permission_name,
            description: new_permission.description,
            is_user: new_permission.is_user.unwrap(),
            is_role: new_permission.is_role.unwrap(),
            is_group: new_permission.is_group.unwrap(),
        }))
    }

    #[oai(
        path = "/permissions/",
        method = "put",
        tag = "ApiPermissionTags::Permission"
    )]
    async fn update_permission_api(
        &self,
        Query(id): Query<String>,
        Json(json): Json<PermissionUpdateRequest>,
        state: Data<&Arc<AppState>>,
        auth: BearerAuthorization,
    ) -> PermissionUpdateResponses {
        // Begin db transaction
        let mut tx = match state.db.begin().await {
            Ok(val) => val,
            Err(err) => {
                return PermissionUpdateResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.permission",
                        "update_permission_api",
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
                return PermissionUpdateResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.permission",
                        "update_permission_api",
                        "get redis pool connection",
                        &err.to_string(),
                    ),
                ))
            }
        };

        // Validate user token
        let jwt_token = auth.0.token;
        let user = match get_user_from_token(&mut tx, &mut redis_conn, jwt_token.clone()).await {
            Ok(val) => val,
            Err(err) => {
                return PermissionUpdateResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.permission",
                        "update_permission_api",
                        "get user from token",
                        &err.to_string(),
                    ),
                ))
            }
        };
        if user.is_none() {
            return PermissionUpdateResponses::Unauthorized(Json(UnauthorizedResponse::default()));
        }
        let request_user = user.unwrap();

        // get detail permission
        let id = match Uuid::parse_str(&id) {
            Ok(val) => val,
            Err(_) => {
                return PermissionUpdateResponses::NotFound(Json(NotFoundResponse {
                    message: format!("permission with id = {} not found", id),
                }))
            }
        };

        let data = match get_permission_by_id(&mut tx, &id).await {
            Ok(val) => val,
            Err(err) => {
                return PermissionUpdateResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.permission",
                        "update_permission_api",
                        "get_permission_by_id",
                        &err.to_string(),
                    ),
                ))
            }
        };
        if data.is_none() {
            return PermissionUpdateResponses::NotFound(Json(NotFoundResponse {
                message: format!("permission with id = {} not found", id),
            }));
        }
        // Validate json request
        let mut permission_attributes: Vec<PermissionAttribute> = vec![];
        for item in json.permission_attribute_ids {
            let permission_attribute_id = match Uuid::parse_str(&item) {
                Ok(val) => val,
                Err(_) => {
                    return PermissionUpdateResponses::BadRequest(Json(BadRequestResponse {
                        message: format!("permission attribute id = {} not found", item),
                    }));
                }
            };
            let permission_attribute =
                match get_permission_attribute_by_id(&mut tx, &permission_attribute_id).await {
                    Ok(val) => val,
                    Err(err) => {
                        return PermissionUpdateResponses::InternalServerError(Json(
                            InternalServerErrorResponse::new(
                                "route.permission",
                                "create_permission_api",
                                "get_permission_attribute_by_id",
                                &err.to_string(),
                            ),
                        ))
                    }
                };
            if permission_attribute.is_none() {
                return PermissionUpdateResponses::BadRequest(Json(BadRequestResponse {
                    message: format!("permission attribute id = {} not found", item),
                }));
            }
            permission_attributes.push(permission_attribute.unwrap());
        }
        // Update permission
        let mut data = data.unwrap();
        let now = Local::now().fixed_offset();
        data.permission_name = json.permission_name;
        data.description = json.description;
        data.is_user = Some(json.is_user);
        data.is_role = Some(json.is_role);
        data.is_group = Some(json.is_group);
        data.updated_by = Some(request_user.id);
        data.updated_date = Some(now);
        if let Err(err) = update_permission(&mut tx, &data).await {
            return PermissionUpdateResponses::InternalServerError(Json(
                InternalServerErrorResponse::new(
                    "route.permission",
                    "update_permission_api",
                    "update_permission",
                    &err.to_string(),
                ),
            ));
        }
        if let Err(err) =
            update_permssion_attribute_list_by_permission(&mut tx, &data, permission_attributes)
                .await
        {
            return PermissionUpdateResponses::InternalServerError(Json(
                InternalServerErrorResponse::new(
                    "route.permission",
                    "update_permission_api",
                    "update_permssion_attribute_list_by_permission",
                    &err.to_string(),
                ),
            ));
        }
        if let Err(err) = tx.commit().await {
            return PermissionUpdateResponses::InternalServerError(Json(
                InternalServerErrorResponse::new(
                    "route.permission",
                    "update_permission_api",
                    "commit transaction",
                    &err.to_string(),
                ),
            ));
        }

        PermissionUpdateResponses::Ok(Json(PermissionUpdateResponse {
            id: data.id.to_string(),
            permission_name: data.permission_name,
            description: data.description,
            is_user: data.is_user.unwrap_or(false),
            is_role: data.is_role.unwrap_or(false),
            is_group: data.is_group.unwrap_or(false),
        }))
    }

    #[oai(
        path = "/permissions/",
        method = "delete",
        tag = "ApiPermissionTags::Permission"
    )]
    async fn delete_permission_api(
        &self,
        Query(id): Query<String>,
        state: Data<&Arc<AppState>>,
        auth: BearerAuthorization,
    ) -> PermissionDeleteResponses {
        // Begin db transaction
        let mut tx = match state.db.begin().await {
            Ok(val) => val,
            Err(err) => {
                return PermissionDeleteResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.permission",
                        "delete_permission_api",
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
                return PermissionDeleteResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.permission",
                        "delete_permission_api",
                        "get redis pool connection",
                        &err.to_string(),
                    ),
                ))
            }
        };

        // Validate user token
        let jwt_token = auth.0.token;
        let user = match get_user_from_token(&mut tx, &mut redis_conn, jwt_token.clone()).await {
            Ok(val) => val,
            Err(err) => {
                return PermissionDeleteResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.permission",
                        "delete_permission_api",
                        "get user from token",
                        &err.to_string(),
                    ),
                ))
            }
        };
        if user.is_none() {
            return PermissionDeleteResponses::Unauthorized(Json(UnauthorizedResponse::default()));
        }

        // get detail permission
        let id = match Uuid::parse_str(&id) {
            Ok(val) => val,
            Err(_) => {
                return PermissionDeleteResponses::NotFound(Json(NotFoundResponse {
                    message: format!("permission with id = {} not found", id),
                }))
            }
        };

        let data = match get_permission_by_id(&mut tx, &id).await {
            Ok(val) => val,
            Err(err) => {
                return PermissionDeleteResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.permission",
                        "delete_permission_api",
                        "get_permission_by_id",
                        &err.to_string(),
                    ),
                ))
            }
        };
        if data.is_none() {
            return PermissionDeleteResponses::NotFound(Json(NotFoundResponse {
                message: format!("permission with id = {} not found", id),
            }));
        }
        let data = data.unwrap();
        if let Err(err) = delete_permission(&mut tx, &data).await {
            return PermissionDeleteResponses::InternalServerError(Json(
                InternalServerErrorResponse::new(
                    "route.permission",
                    "delete_permission_api",
                    "delete_permission",
                    &err.to_string(),
                ),
            ));
        }
        if let Err(err) = tx.commit().await {
            return PermissionDeleteResponses::InternalServerError(Json(
                InternalServerErrorResponse::new(
                    "route.permission",
                    "delete_permission_api",
                    "commit transaction",
                    &err.to_string(),
                ),
            ));
        }
        PermissionDeleteResponses::NoContent
    }
}
