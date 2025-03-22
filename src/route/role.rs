use std::sync::Arc;

use poem::web::Data;
use poem_openapi::{param::Query, payload::Json, OpenApi, Tags};
use uuid::Uuid;

use crate::{
    core::{
        security::{get_user_from_token, BearerAuthorization},
        utils::datetime_to_string_opt,
    },
    model::user::User,
    repository::{
        role::{
            create_role, get_all_role, get_dropdown_role, get_role_by_id, paginate_role,
            soft_delete_role, update_role,
        },
        user::get_user_by_id,
    },
    schema::{
        common::{
            InternalServerErrorResponse, NotFoundResponse, PaginateResponse, UnauthorizedResponse,
        },
        role::{
            DetailRolePagination, PaginateRoleResponses, RoleAllResponse, RoleAllResponses,
            RoleCreateRequest, RoleCreateResponse, RoleCreateResponses, RoleDeleteResponses,
            RoleDetailResponses, RoleDetailSuccessResponse, RoleDetailUser, RoleDropdownResponse,
            RoleDropdownResponses, RoleUpdateRequest, RoleUpdateResponse, RoleUpdateResponses,
        },
    },
    AppState,
};

#[derive(Tags)]
enum ApiRoleTags {
    Role,
}

pub struct ApiRole;

#[OpenApi]
impl ApiRole {
    #[oai(path = "/role/", method = "get", tag = "ApiRoleTags::Role")]
    async fn paginate_role_api(
        &self,
        Query(page): Query<Option<u32>>,
        Query(page_size): Query<Option<u32>>,
        Query(search): Query<Option<String>>,
        state: Data<&Arc<AppState>>,
        auth: BearerAuthorization,
    ) -> PaginateRoleResponses {
        // Begin db transaction
        let mut tx = match state.db.begin().await {
            Ok(val) => val,
            Err(err) => {
                return PaginateRoleResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.role",
                        "paginate_role_api",
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
                return PaginateRoleResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.role",
                        "paginate_role_api",
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
                return PaginateRoleResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.role",
                        "paginate_role_api",
                        "get user from token",
                        &err.to_string(),
                    ),
                ))
            }
        };
        if user.is_none() {
            return PaginateRoleResponses::Unauthorized(Json(UnauthorizedResponse::default()));
        }
        let page = page.unwrap_or(1);
        let page_size = page_size.unwrap_or(10);
        let (data, counts, page_count) = match paginate_role(&mut tx, page, page_size, search).await
        {
            Ok(val) => val,
            Err(err) => {
                return PaginateRoleResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.role",
                        "paginate_role_api",
                        "paginate_role",
                        &err.to_string(),
                    ),
                ))
            }
        };

        let mut results: Vec<DetailRolePagination> = vec![];
        for item in data {
            let mut created_by: Option<User> = None;
            if let Some(created_by_id) = item.created_by {
                (created_by, _) = match get_user_by_id(&mut tx, &created_by_id).await {
                    Ok(val) => val,
                    Err(err) => {
                        return PaginateRoleResponses::InternalServerError(Json(
                            InternalServerErrorResponse::new(
                                "route.role",
                                "paginate_role_api",
                                "get created_by",
                                &err.to_string(),
                            ),
                        ))
                    }
                };
            }
            let mut updated_by: Option<User> = None;
            if let Some(updated_by_id) = item.updated_by {
                (updated_by, _) = match get_user_by_id(&mut tx, &updated_by_id).await {
                    Ok(val) => val,
                    Err(err) => {
                        return PaginateRoleResponses::InternalServerError(Json(
                            InternalServerErrorResponse::new(
                                "route.role",
                                "paginate_role_api",
                                "get updated_by",
                                &err.to_string(),
                            ),
                        ))
                    }
                };
            }
            results.push(DetailRolePagination {
                id: item.id.to_string(),
                role_name: item.role_name,
                description: item.description,
                is_active: item.is_active,
                created_by: match created_by {
                    Some(val) => Some(RoleDetailUser {
                        id: val.id.to_string(),
                        user_name: val.user_name,
                    }),
                    None => None,
                },
                updated_by: match updated_by {
                    Some(val) => Some(RoleDetailUser {
                        id: val.id.to_string(),
                        user_name: val.user_name,
                    }),
                    None => None,
                },
                created_date: datetime_to_string_opt(item.created_date),
                updated_date: datetime_to_string_opt(item.updated_date),
            });
        }

        PaginateRoleResponses::Ok(Json(PaginateResponse {
            counts,
            page,
            page_count,
            page_size,
            results,
        }))
    }

    #[oai(path = "/role/all/", method = "get", tag = "ApiRoleTags::Role")]
    async fn get_all_role_api(
        &self,
        state: Data<&Arc<AppState>>,
        auth: BearerAuthorization,
    ) -> RoleAllResponses {
        // Begin db transaction
        let mut tx = match state.db.begin().await {
            Ok(val) => val,
            Err(err) => {
                return RoleAllResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.role",
                        "get_all_role_api",
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
                return RoleAllResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.role",
                        "get_all_role_api",
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
                return RoleAllResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.role",
                        "get_all_role_api",
                        "get user from token",
                        &err.to_string(),
                    ),
                ))
            }
        };
        if user.is_none() {
            return RoleAllResponses::Unauthorized(Json(UnauthorizedResponse::default()));
        }

        let data = match get_all_role(&mut tx).await {
            Ok(val) => val,
            Err(err) => {
                return RoleAllResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.role",
                        "get_all_role_api",
                        "get_all_role",
                        &err.to_string(),
                    ),
                ))
            }
        };

        let mut results: Vec<RoleAllResponse> = vec![];
        for item in data {
            let mut created_by: Option<User> = None;
            if let Some(created_by_id) = item.created_by {
                (created_by, _) = match get_user_by_id(&mut tx, &created_by_id).await {
                    Ok(val) => val,
                    Err(err) => {
                        return RoleAllResponses::InternalServerError(Json(
                            InternalServerErrorResponse::new(
                                "route.role",
                                "get_all_role_api",
                                "get created_by",
                                &err.to_string(),
                            ),
                        ))
                    }
                };
            }
            let mut updated_by: Option<User> = None;
            if let Some(updated_by_id) = item.updated_by {
                (updated_by, _) = match get_user_by_id(&mut tx, &updated_by_id).await {
                    Ok(val) => val,
                    Err(err) => {
                        return RoleAllResponses::InternalServerError(Json(
                            InternalServerErrorResponse::new(
                                "route.role",
                                "get_all_role_api",
                                "get updated_by",
                                &err.to_string(),
                            ),
                        ))
                    }
                };
            }
            results.push(RoleAllResponse {
                id: item.id.to_string(),
                role_name: item.role_name,
                description: item.description,
                is_active: item.is_active,
                created_by: match created_by {
                    Some(val) => Some(RoleDetailUser {
                        id: val.id.to_string(),
                        user_name: val.user_name,
                    }),
                    None => None,
                },
                updated_by: match updated_by {
                    Some(val) => Some(RoleDetailUser {
                        id: val.id.to_string(),
                        user_name: val.user_name,
                    }),
                    None => None,
                },
                created_date: datetime_to_string_opt(item.created_date),
                updated_date: datetime_to_string_opt(item.updated_date),
            });
        }

        RoleAllResponses::Ok(Json(results))
    }

    #[oai(path = "/role/dropdown/", method = "get", tag = "ApiRoleTags::Role")]
    async fn get_dropdown_role_api(
        &self,
        Query(limit): Query<Option<u32>>,
        Query(search): Query<Option<String>>,
        state: Data<&Arc<AppState>>,
        auth: BearerAuthorization,
    ) -> RoleDropdownResponses {
        // Begin db transaction
        let mut tx = match state.db.begin().await {
            Ok(val) => val,
            Err(err) => {
                return RoleDropdownResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.role",
                        "get_dropdown_role_api",
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
                return RoleDropdownResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.role",
                        "get_dropdown_role_api",
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
                return RoleDropdownResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.role",
                        "get_dropdown_role_api",
                        "get user from token",
                        &err.to_string(),
                    ),
                ))
            }
        };
        if user.is_none() {
            return RoleDropdownResponses::Unauthorized(Json(UnauthorizedResponse::default()));
        }

        let data = match get_dropdown_role(&mut tx, limit, search).await {
            Ok(val) => val,
            Err(err) => {
                return RoleDropdownResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.role",
                        "get_dropdown_role_api",
                        "get_dropdown_role",
                        &err.to_string(),
                    ),
                ))
            }
        };

        RoleDropdownResponses::Ok(Json(
            data.iter()
                .map(|x| RoleDropdownResponse {
                    id: x.id.to_string(),
                    role_name: x.role_name.clone(),
                })
                .collect(),
        ))
    }

    #[oai(path = "/role/detail/", method = "get", tag = "ApiRoleTags::Role")]
    async fn get_detail_role_api(
        &self,
        Query(id): Query<String>,
        state: Data<&Arc<AppState>>,
        auth: BearerAuthorization,
    ) -> RoleDetailResponses {
        // Begin db transaction
        let mut tx = match state.db.begin().await {
            Ok(val) => val,
            Err(err) => {
                return RoleDetailResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.role",
                        "get_detail_role_api",
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
                return RoleDetailResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.role",
                        "get_detail_role_api",
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
                return RoleDetailResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.role",
                        "get_detail_role_api",
                        "get user from token",
                        &err.to_string(),
                    ),
                ))
            }
        };
        if user.is_none() {
            return RoleDetailResponses::Unauthorized(Json(UnauthorizedResponse::default()));
        }

        let id = match Uuid::parse_str(&id) {
            Ok(val) => val,
            Err(_) => {
                return RoleDetailResponses::NotFound(Json(NotFoundResponse {
                    message: format!("role with id = {} not found", id),
                }))
            }
        };

        let data = match get_role_by_id(&mut tx, &id).await {
            Ok(val) => val,
            Err(err) => {
                return RoleDetailResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.role",
                        "get_detail_role_api",
                        "get_role_by_id",
                        &err.to_string(),
                    ),
                ))
            }
        };
        if data.is_none() {
            return RoleDetailResponses::NotFound(Json(NotFoundResponse {
                message: format!("role with id = {} not found", id),
            }));
        }
        let data = data.unwrap();
        let mut created_by: Option<User> = None;
        if let Some(created_by_id) = data.created_by {
            (created_by, _) = match get_user_by_id(&mut tx, &created_by_id).await {
                Ok(val) => val,
                Err(err) => {
                    return RoleDetailResponses::InternalServerError(Json(
                        InternalServerErrorResponse::new(
                            "route.role",
                            "get_detail_role_api",
                            "get created_by",
                            &err.to_string(),
                        ),
                    ))
                }
            };
        }
        let mut updated_by: Option<User> = None;
        if let Some(updated_by_id) = data.updated_by {
            (updated_by, _) = match get_user_by_id(&mut tx, &updated_by_id).await {
                Ok(val) => val,
                Err(err) => {
                    return RoleDetailResponses::InternalServerError(Json(
                        InternalServerErrorResponse::new(
                            "route.role",
                            "get_detail_role_api",
                            "get updated_by",
                            &err.to_string(),
                        ),
                    ))
                }
            };
        }
        RoleDetailResponses::Ok(Json(RoleDetailSuccessResponse {
            id: data.id.to_string(),
            role_name: data.role_name,
            description: data.description,
            is_active: data.is_active,
            created_date: datetime_to_string_opt(data.created_date),
            updated_date: datetime_to_string_opt(data.updated_date),
            created_by: created_by.map(|x| RoleDetailUser {
                id: x.id.to_string(),
                user_name: x.user_name,
            }),
            updated_by: updated_by.map(|x| RoleDetailUser {
                id: x.id.to_string(),
                user_name: x.user_name,
            }),
        }))
    }

    #[oai(path = "/role/", method = "post", tag = "ApiRoleTags::Role")]
    async fn create_role_api(
        &self,
        Json(json): Json<RoleCreateRequest>,
        state: Data<&Arc<AppState>>,
        auth: BearerAuthorization,
    ) -> RoleCreateResponses {
        // Begin db transaction
        let mut tx = match state.db.begin().await {
            Ok(val) => val,
            Err(err) => {
                return RoleCreateResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.role",
                        "create_role_api",
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
                return RoleCreateResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.role",
                        "create_role_api",
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
                    return RoleCreateResponses::InternalServerError(Json(
                        InternalServerErrorResponse::new(
                            "route.role",
                            "create_role_api",
                            "get user from token",
                            &err.to_string(),
                        ),
                    ))
                }
            };
        if request_user.is_none() {
            return RoleCreateResponses::Unauthorized(Json(UnauthorizedResponse::default()));
        }
        let request_user = request_user.unwrap();

        let new_role = match create_role(
            &mut tx,
            None,
            json.role_name,
            json.description,
            json.is_active,
            request_user,
            None,
        )
        .await
        {
            Ok(val) => val,
            Err(err) => {
                return RoleCreateResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.role",
                        "create_role_api",
                        "create_role",
                        &err.to_string(),
                    ),
                ))
            }
        };
        if let Err(err) = tx.commit().await {
            return RoleCreateResponses::InternalServerError(Json(
                InternalServerErrorResponse::new(
                    "route.role",
                    "create_role_api",
                    "commit transaction",
                    &err.to_string(),
                ),
            ));
        }
        RoleCreateResponses::Ok(Json(RoleCreateResponse {
            id: new_role.id.to_string(),
            role_name: new_role.role_name,
            description: new_role.description,
            is_active: new_role.is_active,
        }))
    }

    #[oai(path = "/role/", method = "put", tag = "ApiRoleTags::Role")]
    async fn update_role_api(
        &self,
        Query(id): Query<String>,
        Json(json): Json<RoleUpdateRequest>,
        state: Data<&Arc<AppState>>,
        auth: BearerAuthorization,
    ) -> RoleUpdateResponses {
        // Begin db transaction
        let mut tx = match state.db.begin().await {
            Ok(val) => val,
            Err(err) => {
                return RoleUpdateResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.role",
                        "update_role_api",
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
                return RoleUpdateResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.role",
                        "update_role_api",
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
                    return RoleUpdateResponses::InternalServerError(Json(
                        InternalServerErrorResponse::new(
                            "route.role",
                            "update_role_api",
                            "get user from token",
                            &err.to_string(),
                        ),
                    ))
                }
            };
        if request_user.is_none() {
            return RoleUpdateResponses::Unauthorized(Json(UnauthorizedResponse::default()));
        }
        let request_user = request_user.unwrap();

        let id = match Uuid::parse_str(&id) {
            Ok(val) => val,
            Err(_) => {
                return RoleUpdateResponses::NotFound(Json(NotFoundResponse {
                    message: format!("role with id = {} not found", id),
                }))
            }
        };

        let data = match get_role_by_id(&mut tx, &id).await {
            Ok(val) => val,
            Err(err) => {
                return RoleUpdateResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.role",
                        "update_role_api",
                        "get_role_by_id",
                        &err.to_string(),
                    ),
                ))
            }
        };
        if data.is_none() {
            return RoleUpdateResponses::NotFound(Json(NotFoundResponse {
                message: format!("role with id = {} not found", id),
            }));
        }
        let mut data = data.unwrap();

        if let Err(err) = update_role(
            &mut tx,
            &mut data,
            json.role_name,
            json.description,
            json.is_active,
            request_user,
            None,
        )
        .await
        {
            return RoleUpdateResponses::InternalServerError(Json(
                InternalServerErrorResponse::new(
                    "route.role",
                    "update_role_api",
                    "update_role",
                    &err.to_string(),
                ),
            ));
        }

        if let Err(err) = tx.commit().await {
            return RoleUpdateResponses::InternalServerError(Json(
                InternalServerErrorResponse::new(
                    "route.role",
                    "update_role_api",
                    "commit transaction",
                    &err.to_string(),
                ),
            ));
        }
        RoleUpdateResponses::Ok(Json(RoleUpdateResponse {
            id: data.id.to_string(),
            role_name: data.role_name,
            description: data.description,
            is_active: data.is_active,
        }))
    }

    #[oai(path = "/role/", method = "delete", tag = "ApiRoleTags::Role")]
    async fn delete_role_api(
        &self,
        Query(id): Query<String>,
        state: Data<&Arc<AppState>>,
        auth: BearerAuthorization,
    ) -> RoleDeleteResponses {
        // Begin db transaction
        let mut tx = match state.db.begin().await {
            Ok(val) => val,
            Err(err) => {
                return RoleDeleteResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.role",
                        "delete_role_api",
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
                return RoleDeleteResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.role",
                        "delete_role_api",
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
                    return RoleDeleteResponses::InternalServerError(Json(
                        InternalServerErrorResponse::new(
                            "route.role",
                            "delete_role_api",
                            "get user from token",
                            &err.to_string(),
                        ),
                    ))
                }
            };
        if request_user.is_none() {
            return RoleDeleteResponses::Unauthorized(Json(UnauthorizedResponse::default()));
        }
        let request_user = request_user.unwrap();

        let id = match Uuid::parse_str(&id) {
            Ok(val) => val,
            Err(_) => {
                return RoleDeleteResponses::NotFound(Json(NotFoundResponse {
                    message: format!("role with id = {} not found", id),
                }))
            }
        };

        let data = match get_role_by_id(&mut tx, &id).await {
            Ok(val) => val,
            Err(err) => {
                return RoleDeleteResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.role",
                        "delete_role_api",
                        "get_role_by_id",
                        &err.to_string(),
                    ),
                ))
            }
        };
        if data.is_none() {
            return RoleDeleteResponses::NotFound(Json(NotFoundResponse {
                message: format!("role with id = {} not found", id),
            }));
        }
        let mut data = data.unwrap();

        if let Err(err) = soft_delete_role(&mut tx, &mut data, request_user, None).await {
            return RoleDeleteResponses::InternalServerError(Json(
                InternalServerErrorResponse::new(
                    "route.role",
                    "delete_role_api",
                    "soft_delete_role",
                    &err.to_string(),
                ),
            ));
        }

        if let Err(err) = tx.commit().await {
            return RoleDeleteResponses::InternalServerError(Json(
                InternalServerErrorResponse::new(
                    "route.role",
                    "delete_role_api",
                    "commit transaction",
                    &err.to_string(),
                ),
            ));
        }
        RoleDeleteResponses::NoContent
    }
}
