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
        group::{
            create_group, get_all_group, get_dropdown_group, get_group_by_id, paginate_group,
            soft_delete_group, update_group,
        },
        user::get_user_by_id,
    },
    schema::{
        common::{
            InternalServerErrorResponse, NotFoundResponse, PaginateResponse, UnauthorizedResponse,
        },
        group::{
            DetailGroupPagination, GroupAllResponse, GroupAllResponses, GroupCreateRequest,
            GroupCreateResponse, GroupCreateResponses, GroupDeleteResponses, GroupDetailResponses,
            GroupDetailSuccessResponse, GroupDetailUser, GroupDropdownResponse,
            GroupDropdownResponses, GroupUpdateRequest, GroupUpdateResponse, GroupUpdateResponses,
            PaginateGroupResponses,
        },
    },
    AppState,
};

#[derive(Tags)]
enum ApiGroupTags {
    Group,
}

pub struct ApiGroup;

#[OpenApi]
impl ApiGroup {
    #[oai(path = "/group/", method = "get", tag = "ApiGroupTags::Group")]
    async fn paginate_group_api(
        &self,
        Query(page): Query<Option<u32>>,
        Query(page_size): Query<Option<u32>>,
        Query(search): Query<Option<String>>,
        state: Data<&Arc<AppState>>,
        auth: BearerAuthorization,
    ) -> PaginateGroupResponses {
        // Begin db transaction
        let mut tx = match state.db.begin().await {
            Ok(val) => val,
            Err(err) => {
                return PaginateGroupResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.group",
                        "paginate_group_api",
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
                return PaginateGroupResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.group",
                        "paginate_group_api",
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
                return PaginateGroupResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.group",
                        "paginate_group_api",
                        "get user from token",
                        &err.to_string(),
                    ),
                ))
            }
        };
        if user.is_none() {
            return PaginateGroupResponses::Unauthorized(Json(UnauthorizedResponse::default()));
        }
        let page = page.unwrap_or(1);
        let page_size = page_size.unwrap_or(10);
        let (data, counts, page_count) =
            match paginate_group(&mut tx, page, page_size, search).await {
                Ok(val) => val,
                Err(err) => {
                    return PaginateGroupResponses::InternalServerError(Json(
                        InternalServerErrorResponse::new(
                            "route.group",
                            "paginate_group_api",
                            "paginate_group",
                            &err.to_string(),
                        ),
                    ))
                }
            };

        let mut results: Vec<DetailGroupPagination> = vec![];
        for item in data {
            let mut created_by: Option<User> = None;
            if let Some(created_by_id) = item.created_by {
                (created_by, _) = match get_user_by_id(&mut tx, &created_by_id, None).await {
                    Ok(val) => val,
                    Err(err) => {
                        return PaginateGroupResponses::InternalServerError(Json(
                            InternalServerErrorResponse::new(
                                "route.group",
                                "paginate_group_api",
                                "get created_by",
                                &err.to_string(),
                            ),
                        ))
                    }
                };
            }
            let mut updated_by: Option<User> = None;
            if let Some(updated_by_id) = item.updated_by {
                (updated_by, _) = match get_user_by_id(&mut tx, &updated_by_id, None).await {
                    Ok(val) => val,
                    Err(err) => {
                        return PaginateGroupResponses::InternalServerError(Json(
                            InternalServerErrorResponse::new(
                                "route.group",
                                "paginate_group_api",
                                "get updated_by",
                                &err.to_string(),
                            ),
                        ))
                    }
                };
            }
            results.push(DetailGroupPagination {
                id: item.id.to_string(),
                group_name: item.group_name,
                description: item.description,
                is_active: item.is_active,
                created_by: match created_by {
                    Some(val) => Some(GroupDetailUser {
                        id: val.id.to_string(),
                        user_name: val.user_name,
                    }),
                    None => None,
                },
                updated_by: match updated_by {
                    Some(val) => Some(GroupDetailUser {
                        id: val.id.to_string(),
                        user_name: val.user_name,
                    }),
                    None => None,
                },
                created_date: datetime_to_string_opt(item.created_date),
                updated_date: datetime_to_string_opt(item.updated_date),
            });
        }

        PaginateGroupResponses::Ok(Json(PaginateResponse {
            counts,
            page,
            page_count,
            page_size,
            results,
        }))
    }

    #[oai(path = "/group/all/", method = "get", tag = "ApiGroupTags::Group")]
    async fn get_all_group_api(
        &self,
        state: Data<&Arc<AppState>>,
        auth: BearerAuthorization,
    ) -> GroupAllResponses {
        // Begin db transaction
        let mut tx = match state.db.begin().await {
            Ok(val) => val,
            Err(err) => {
                return GroupAllResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.group",
                        "get_all_group_api",
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
                return GroupAllResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.group",
                        "get_all_group_api",
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
                return GroupAllResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.group",
                        "get_all_group_api",
                        "get user from token",
                        &err.to_string(),
                    ),
                ))
            }
        };
        if user.is_none() {
            return GroupAllResponses::Unauthorized(Json(UnauthorizedResponse::default()));
        }

        let data = match get_all_group(&mut tx).await {
            Ok(val) => val,
            Err(err) => {
                return GroupAllResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.group",
                        "get_all_group_api",
                        "get_all_group",
                        &err.to_string(),
                    ),
                ))
            }
        };

        let mut results: Vec<GroupAllResponse> = vec![];
        for item in data {
            let mut created_by: Option<User> = None;
            if let Some(created_by_id) = item.created_by {
                (created_by, _) = match get_user_by_id(&mut tx, &created_by_id, None).await {
                    Ok(val) => val,
                    Err(err) => {
                        return GroupAllResponses::InternalServerError(Json(
                            InternalServerErrorResponse::new(
                                "route.group",
                                "get_all_group_api",
                                "get created_by",
                                &err.to_string(),
                            ),
                        ))
                    }
                };
            }
            let mut updated_by: Option<User> = None;
            if let Some(updated_by_id) = item.updated_by {
                (updated_by, _) = match get_user_by_id(&mut tx, &updated_by_id, None).await {
                    Ok(val) => val,
                    Err(err) => {
                        return GroupAllResponses::InternalServerError(Json(
                            InternalServerErrorResponse::new(
                                "route.group",
                                "get_all_group_api",
                                "get updated_by",
                                &err.to_string(),
                            ),
                        ))
                    }
                };
            }
            results.push(GroupAllResponse {
                id: item.id.to_string(),
                group_name: item.group_name,
                description: item.description,
                is_active: item.is_active,
                created_by: match created_by {
                    Some(val) => Some(GroupDetailUser {
                        id: val.id.to_string(),
                        user_name: val.user_name,
                    }),
                    None => None,
                },
                updated_by: match updated_by {
                    Some(val) => Some(GroupDetailUser {
                        id: val.id.to_string(),
                        user_name: val.user_name,
                    }),
                    None => None,
                },
                created_date: datetime_to_string_opt(item.created_date),
                updated_date: datetime_to_string_opt(item.updated_date),
            });
        }

        GroupAllResponses::Ok(Json(results))
    }

    #[oai(path = "/group/dropdown/", method = "get", tag = "ApiGroupTags::Group")]
    async fn get_dropdown_group_api(
        &self,
        Query(limit): Query<Option<u32>>,
        Query(search): Query<Option<String>>,
        state: Data<&Arc<AppState>>,
        auth: BearerAuthorization,
    ) -> GroupDropdownResponses {
        // Begin db transaction
        let mut tx = match state.db.begin().await {
            Ok(val) => val,
            Err(err) => {
                return GroupDropdownResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.group",
                        "get_dropdown_group_api",
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
                return GroupDropdownResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.group",
                        "get_dropdown_group_api",
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
                return GroupDropdownResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.group",
                        "get_dropdown_group_api",
                        "get user from token",
                        &err.to_string(),
                    ),
                ))
            }
        };
        if user.is_none() {
            return GroupDropdownResponses::Unauthorized(Json(UnauthorizedResponse::default()));
        }

        let data = match get_dropdown_group(&mut tx, limit, search).await {
            Ok(val) => val,
            Err(err) => {
                return GroupDropdownResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.group",
                        "get_dropdown_group_api",
                        "get_dropdown_group",
                        &err.to_string(),
                    ),
                ))
            }
        };

        GroupDropdownResponses::Ok(Json(
            data.iter()
                .map(|x| GroupDropdownResponse {
                    id: x.id.to_string(),
                    group_name: x.group_name.clone(),
                })
                .collect(),
        ))
    }

    #[oai(path = "/group/detail/", method = "get", tag = "ApiGroupTags::Group")]
    async fn get_detail_group_api(
        &self,
        Query(id): Query<String>,
        state: Data<&Arc<AppState>>,
        auth: BearerAuthorization,
    ) -> GroupDetailResponses {
        // Begin db transaction
        let mut tx = match state.db.begin().await {
            Ok(val) => val,
            Err(err) => {
                return GroupDetailResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.group",
                        "get_detail_group_api",
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
                return GroupDetailResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.group",
                        "get_detail_group_api",
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
                return GroupDetailResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.group",
                        "get_detail_group_api",
                        "get user from token",
                        &err.to_string(),
                    ),
                ))
            }
        };
        if user.is_none() {
            return GroupDetailResponses::Unauthorized(Json(UnauthorizedResponse::default()));
        }

        let id = match Uuid::parse_str(&id) {
            Ok(val) => val,
            Err(_) => {
                return GroupDetailResponses::NotFound(Json(NotFoundResponse {
                    message: format!("role with id = {} not found", id),
                }))
            }
        };

        let data = match get_group_by_id(&mut tx, &id).await {
            Ok(val) => val,
            Err(err) => {
                return GroupDetailResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.group",
                        "get_detail_group_api",
                        "get_group_by_id",
                        &err.to_string(),
                    ),
                ))
            }
        };
        if data.is_none() {
            return GroupDetailResponses::NotFound(Json(NotFoundResponse {
                message: format!("role with id = {} not found", id),
            }));
        }
        let data = data.unwrap();
        let mut created_by: Option<User> = None;
        if let Some(created_by_id) = data.created_by {
            (created_by, _) = match get_user_by_id(&mut tx, &created_by_id, None).await {
                Ok(val) => val,
                Err(err) => {
                    return GroupDetailResponses::InternalServerError(Json(
                        InternalServerErrorResponse::new(
                            "route.group",
                            "get_detail_group_api",
                            "get created_by",
                            &err.to_string(),
                        ),
                    ))
                }
            };
        }
        let mut updated_by: Option<User> = None;
        if let Some(updated_by_id) = data.updated_by {
            (updated_by, _) = match get_user_by_id(&mut tx, &updated_by_id, None).await {
                Ok(val) => val,
                Err(err) => {
                    return GroupDetailResponses::InternalServerError(Json(
                        InternalServerErrorResponse::new(
                            "route.group",
                            "get_detail_group_api",
                            "get updated_by",
                            &err.to_string(),
                        ),
                    ))
                }
            };
        }
        GroupDetailResponses::Ok(Json(GroupDetailSuccessResponse {
            id: data.id.to_string(),
            group_name: data.group_name,
            description: data.description,
            is_active: data.is_active,
            created_date: datetime_to_string_opt(data.created_date),
            updated_date: datetime_to_string_opt(data.updated_date),
            created_by: created_by.map(|x| GroupDetailUser {
                id: x.id.to_string(),
                user_name: x.user_name,
            }),
            updated_by: updated_by.map(|x| GroupDetailUser {
                id: x.id.to_string(),
                user_name: x.user_name,
            }),
        }))
    }

    #[oai(path = "/group/", method = "post", tag = "ApiGroupTags::Group")]
    async fn create_group_api(
        &self,
        Json(json): Json<GroupCreateRequest>,
        state: Data<&Arc<AppState>>,
        auth: BearerAuthorization,
    ) -> GroupCreateResponses {
        // Begin db transaction
        let mut tx = match state.db.begin().await {
            Ok(val) => val,
            Err(err) => {
                return GroupCreateResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.group",
                        "create_group_api",
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
                return GroupCreateResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.group",
                        "create_group_api",
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
                    return GroupCreateResponses::InternalServerError(Json(
                        InternalServerErrorResponse::new(
                            "route.group",
                            "create_group_api",
                            "get user from token",
                            &err.to_string(),
                        ),
                    ))
                }
            };
        if request_user.is_none() {
            return GroupCreateResponses::Unauthorized(Json(UnauthorizedResponse::default()));
        }
        let request_user = request_user.unwrap();

        let new_group = match create_group(
            &mut tx,
            None,
            json.group_name,
            json.description,
            json.is_active,
            request_user,
            None,
        )
        .await
        {
            Ok(val) => val,
            Err(err) => {
                return GroupCreateResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.group",
                        "create_group_api",
                        "create_group",
                        &err.to_string(),
                    ),
                ))
            }
        };
        if let Err(err) = tx.commit().await {
            return GroupCreateResponses::InternalServerError(Json(
                InternalServerErrorResponse::new(
                    "route.group",
                    "create_group_api",
                    "commit transaction",
                    &err.to_string(),
                ),
            ));
        }
        GroupCreateResponses::Ok(Json(GroupCreateResponse {
            id: new_group.id.to_string(),
            group_name: new_group.group_name,
            description: new_group.description,
            is_active: new_group.is_active,
        }))
    }

    #[oai(path = "/group/", method = "put", tag = "ApiGroupTags::Group")]
    async fn update_group_api(
        &self,
        Query(id): Query<String>,
        Json(json): Json<GroupUpdateRequest>,
        state: Data<&Arc<AppState>>,
        auth: BearerAuthorization,
    ) -> GroupUpdateResponses {
        // Begin db transaction
        let mut tx = match state.db.begin().await {
            Ok(val) => val,
            Err(err) => {
                return GroupUpdateResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.group",
                        "update_group_api",
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
                return GroupUpdateResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.group",
                        "update_group_api",
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
                    return GroupUpdateResponses::InternalServerError(Json(
                        InternalServerErrorResponse::new(
                            "route.group",
                            "update_group_api",
                            "get user from token",
                            &err.to_string(),
                        ),
                    ))
                }
            };
        if request_user.is_none() {
            return GroupUpdateResponses::Unauthorized(Json(UnauthorizedResponse::default()));
        }
        let request_user = request_user.unwrap();

        let id = match Uuid::parse_str(&id) {
            Ok(val) => val,
            Err(_) => {
                return GroupUpdateResponses::NotFound(Json(NotFoundResponse {
                    message: format!("role with id = {} not found", id),
                }))
            }
        };

        let data = match get_group_by_id(&mut tx, &id).await {
            Ok(val) => val,
            Err(err) => {
                return GroupUpdateResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.group",
                        "update_group_api",
                        "get_group_by_id",
                        &err.to_string(),
                    ),
                ))
            }
        };
        if data.is_none() {
            return GroupUpdateResponses::NotFound(Json(NotFoundResponse {
                message: format!("role with id = {} not found", id),
            }));
        }
        let mut data = data.unwrap();

        if let Err(err) = update_group(
            &mut tx,
            &mut data,
            json.group_name,
            json.description,
            json.is_active,
            request_user,
            None,
        )
        .await
        {
            return GroupUpdateResponses::InternalServerError(Json(
                InternalServerErrorResponse::new(
                    "route.group",
                    "update_group_api",
                    "update_group",
                    &err.to_string(),
                ),
            ));
        }

        if let Err(err) = tx.commit().await {
            return GroupUpdateResponses::InternalServerError(Json(
                InternalServerErrorResponse::new(
                    "route.group",
                    "update_group_api",
                    "commit transaction",
                    &err.to_string(),
                ),
            ));
        }
        GroupUpdateResponses::Ok(Json(GroupUpdateResponse {
            id: data.id.to_string(),
            group_name: data.group_name,
            description: data.description,
            is_active: data.is_active,
        }))
    }

    #[oai(path = "/group/", method = "delete", tag = "ApiGroupTags::Group")]
    async fn delete_group_api(
        &self,
        Query(id): Query<String>,
        state: Data<&Arc<AppState>>,
        auth: BearerAuthorization,
    ) -> GroupDeleteResponses {
        // Begin db transaction
        let mut tx = match state.db.begin().await {
            Ok(val) => val,
            Err(err) => {
                return GroupDeleteResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.group",
                        "delete_group_api",
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
                return GroupDeleteResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.group",
                        "delete_group_api",
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
                    return GroupDeleteResponses::InternalServerError(Json(
                        InternalServerErrorResponse::new(
                            "route.group",
                            "delete_group_api",
                            "get user from token",
                            &err.to_string(),
                        ),
                    ))
                }
            };
        if request_user.is_none() {
            return GroupDeleteResponses::Unauthorized(Json(UnauthorizedResponse::default()));
        }
        let request_user = request_user.unwrap();

        let id = match Uuid::parse_str(&id) {
            Ok(val) => val,
            Err(_) => {
                return GroupDeleteResponses::NotFound(Json(NotFoundResponse {
                    message: format!("role with id = {} not found", id),
                }))
            }
        };

        let data = match get_group_by_id(&mut tx, &id).await {
            Ok(val) => val,
            Err(err) => {
                return GroupDeleteResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.group",
                        "delete_group_api",
                        "get_group_by_id",
                        &err.to_string(),
                    ),
                ))
            }
        };
        if data.is_none() {
            return GroupDeleteResponses::NotFound(Json(NotFoundResponse {
                message: format!("role with id = {} not found", id),
            }));
        }
        let mut data = data.unwrap();

        if let Err(err) = soft_delete_group(&mut tx, &mut data, request_user, None).await {
            return GroupDeleteResponses::InternalServerError(Json(
                InternalServerErrorResponse::new(
                    "route.group",
                    "delete_group_api",
                    "soft_delete_group",
                    &err.to_string(),
                ),
            ));
        }

        if let Err(err) = tx.commit().await {
            return GroupDeleteResponses::InternalServerError(Json(
                InternalServerErrorResponse::new(
                    "route.group",
                    "delete_group_api",
                    "commit transaction",
                    &err.to_string(),
                ),
            ));
        }
        GroupDeleteResponses::NoContent
    }
}
