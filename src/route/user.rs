use std::sync::Arc;

use chrono::Local;
use poem::web::Data;
use poem_openapi::{param::Query, payload::Json, OpenApi, Tags};
use uuid::Uuid;

use crate::{
    core::{
        security::{get_user_from_token, hash_password, BearerAuthorization},
        utils::datetime_to_string_opt,
    },
    model::{
        group::Group, role::Role, user::User, user_group_roles::UserGroupRoles,
        user_profile::UserProfile,
    },
    repository::{
        group::get_group_by_id,
        role::get_role_by_id,
        user::{
            create_user, get_all_user, get_user_by_id, get_user_group_roles_by_user,
            upsert_user_group_roles,
        },
    },
    schema::{
        common::{
            BadRequestResponse, InternalServerErrorResponse, NotFoundResponse, PaginateResponse,
            UnauthorizedResponse,
        },
        user::{
            AddUserGroupRoleRequest, AddUserGroupRoleResponses, ChangeStatusRequest,
            ChangeStatusResponses, DeleteUserGroupRoleResponses, DetailCreatedOrUpdatedUser,
            DetailGroup, DetailGroupRole, DetailRole, DetailUser, DetailUserProfile,
            GetAllUserResponses, GetPaginateUserResponses, ResetPasswordRequest,
            ResetPasswordResponses, UserCreateRequest, UserCreateResponse, UserCreateResponses,
            UserDeleteResponses, UserDetailResponse, UserDetailResponses, UserUpdateRequest,
            UserUpdateResponses,
        },
    },
    AppState,
};

#[derive(Tags)]
enum ApiUserTags {
    User,
}

pub struct ApiUser;

#[OpenApi]
impl ApiUser {
    #[oai(path = "/user/", method = "get", tag = "ApiUserTags::User")]
    async fn get_paginate_user_api(
        &self,
        Query(page): Query<Option<u32>>,
        Query(page_size): Query<Option<u32>>,
        Query(search): Query<Option<String>>,
        state: Data<&Arc<AppState>>,
        auth: BearerAuthorization,
    ) -> GetPaginateUserResponses {
        // Begin db transaction
        let mut tx = match state.db.begin().await {
            Ok(val) => val,
            Err(err) => {
                return GetPaginateUserResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.user",
                        "get_paginate_user_api",
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
                return GetPaginateUserResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.user",
                        "get_paginate_user_api",
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
                    return GetPaginateUserResponses::InternalServerError(Json(
                        InternalServerErrorResponse::new(
                            "route.user",
                            "get_paginate_user_api",
                            "get user from token",
                            &err.to_string(),
                        ),
                    ))
                }
            };
        if request_user.is_none() {
            return GetPaginateUserResponses::Unauthorized(Json(UnauthorizedResponse::default()));
        }

        let page = page.unwrap_or(1);
        let page_size = page_size.unwrap_or(10);
        let (data, counts, page_count) =
            match get_all_user(&mut tx, page, page_size, search, None).await {
                Ok(val) => val,
                Err(err) => {
                    return GetPaginateUserResponses::InternalServerError(Json(
                        InternalServerErrorResponse::new(
                            "route.user",
                            "get_paginate_user_api",
                            "get_all_user",
                            &err.to_string(),
                        ),
                    ))
                }
            };

        let mut results: Vec<DetailUser> = vec![];
        for item in data {
            let mut created_by: Option<User> = None;
            if item.created_by.is_some() {
                (created_by, _) =
                    match get_user_by_id(&mut tx, &item.created_by.unwrap(), None).await {
                        Ok(val) => val,
                        Err(err) => {
                            return GetPaginateUserResponses::InternalServerError(Json(
                                InternalServerErrorResponse::new(
                                    "route.user",
                                    "get_paginate_user_api",
                                    "get_user_detail for created_by",
                                    &err.to_string(),
                                ),
                            ))
                        }
                    };
            }
            results.push(DetailUser {
                id: item.id.to_string(),
                user_name: item.user_name,
                is_active: item.is_active,
                is_2faenabled: item.is_2faenabled,
                created_date: datetime_to_string_opt(item.created_date),
                updated_date: datetime_to_string_opt(item.updated_date),
                created_by: created_by.map(|x| DetailCreatedOrUpdatedUser {
                    id: x.id.to_string(),
                    user_name: x.user_name,
                }),
            });
        }

        GetPaginateUserResponses::Ok(Json(PaginateResponse {
            counts,
            page,
            page_count,
            page_size,
            results,
        }))
    }

    #[oai(path = "/user/all/", method = "get", tag = "ApiUserTags::User")]
    async fn get_all_user_api(
        &self,
        Query(page): Query<Option<u32>>,
        Query(page_size): Query<Option<u32>>,
        Query(search): Query<Option<String>>,
        state: Data<&Arc<AppState>>,
        auth: BearerAuthorization,
    ) -> GetAllUserResponses {
        // Begin db transaction
        let mut tx = match state.db.begin().await {
            Ok(val) => val,
            Err(err) => {
                return GetAllUserResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.user",
                        "get_all_user_api",
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
                return GetAllUserResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.user",
                        "get_all_user_api",
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
                    return GetAllUserResponses::InternalServerError(Json(
                        InternalServerErrorResponse::new(
                            "route.user",
                            "get_all_user_api",
                            "get user from token",
                            &err.to_string(),
                        ),
                    ))
                }
            };
        if request_user.is_none() {
            return GetAllUserResponses::Unauthorized(Json(UnauthorizedResponse::default()));
        }

        let page = page.unwrap_or(1);
        let page_size = page_size.unwrap_or(10);
        let (data, counts, page_count) =
            match get_all_user(&mut tx, page, page_size, search, None).await {
                Ok(val) => val,
                Err(err) => {
                    return GetAllUserResponses::InternalServerError(Json(
                        InternalServerErrorResponse::new(
                            "route.user",
                            "get_all_user_api",
                            "get_all_user",
                            &err.to_string(),
                        ),
                    ))
                }
            };

        let mut results: Vec<DetailUser> = vec![];
        for item in data {
            let mut created_by: Option<User> = None;
            if item.created_by.is_some() {
                (created_by, _) =
                    match get_user_by_id(&mut tx, &item.created_by.unwrap(), None).await {
                        Ok(val) => val,
                        Err(err) => {
                            return GetAllUserResponses::InternalServerError(Json(
                                InternalServerErrorResponse::new(
                                    "route.user",
                                    "get_all_user_api",
                                    "get_user_detail for created_by",
                                    &err.to_string(),
                                ),
                            ))
                        }
                    };
            }
            results.push(DetailUser {
                id: item.id.to_string(),
                user_name: item.user_name,
                is_active: item.is_active,
                is_2faenabled: item.is_2faenabled,
                created_date: datetime_to_string_opt(item.created_date),
                updated_date: datetime_to_string_opt(item.updated_date),
                created_by: created_by.map(|x| DetailCreatedOrUpdatedUser {
                    id: x.id.to_string(),
                    user_name: x.user_name,
                }),
            });
        }

        GetAllUserResponses::Ok(Json(PaginateResponse {
            counts,
            page,
            page_count,
            page_size,
            results,
        }))
    }

    #[oai(path = "/user/detail/", method = "get", tag = "ApiUserTags::User")]
    async fn user_detail_api(
        &self,
        Query(id): Query<String>,
        state: Data<&Arc<AppState>>,
        auth: BearerAuthorization,
    ) -> UserDetailResponses {
        // Begin db transaction
        let mut tx = match state.db.begin().await {
            Ok(val) => val,
            Err(err) => {
                return UserDetailResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.user",
                        "user_detail_api",
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
                return UserDetailResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.user",
                        "user_detail_api",
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
                    return UserDetailResponses::InternalServerError(Json(
                        InternalServerErrorResponse::new(
                            "route.user",
                            "user_detail_api",
                            "get user from token",
                            &err.to_string(),
                        ),
                    ))
                }
            };
        if request_user.is_none() {
            return UserDetailResponses::Unauthorized(Json(UnauthorizedResponse::default()));
        }

        let id = match Uuid::parse_str(&id) {
            Ok(val) => val,
            Err(_) => {
                return UserDetailResponses::NotFound(Json(NotFoundResponse {
                    message: format!("user with id = {} not found", &id),
                }))
            }
        };
        let (user, user_profile) = match get_user_by_id(&mut tx, &id, None).await {
            Ok(val) => val,
            Err(err) => {
                return UserDetailResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.user",
                        "user_detail_api",
                        "get_user_by_id",
                        &err.to_string(),
                    ),
                ))
            }
        };
        if user.is_none() {
            return UserDetailResponses::NotFound(Json(NotFoundResponse {
                message: format!("user with id = {} not found", &id),
            }));
        }
        let user = user.unwrap();
        let mut created_by: Option<User> = None;
        if user.created_by.is_some() {
            let (x, _) = match get_user_by_id(&mut tx, &user.created_by.unwrap(), None).await {
                Ok(val) => val,
                Err(err) => {
                    return UserDetailResponses::InternalServerError(Json(
                        InternalServerErrorResponse::new(
                            "route.user",
                            "user_detail_api",
                            "get created_by user",
                            &err.to_string(),
                        ),
                    ))
                }
            };
            created_by = x
        }
        let mut updated_by: Option<User> = None;
        if user.updated_by.is_some() {
            let (x, _) = match get_user_by_id(&mut tx, &user.updated_by.unwrap(), None).await {
                Ok(val) => val,
                Err(err) => {
                    return UserDetailResponses::InternalServerError(Json(
                        InternalServerErrorResponse::new(
                            "route.user",
                            "user_detail_api",
                            "get updated_by user",
                            &err.to_string(),
                        ),
                    ))
                }
            };
            updated_by = x
        }

        let user_group_roles = match get_user_group_roles_by_user(&mut tx, &user).await {
            Ok(val) => val,
            Err(err) => {
                return UserDetailResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.user",
                        "user_detail_api",
                        "get_user_group_roles_by_user",
                        &err.to_string(),
                    ),
                ))
            }
        };
        let mut group_roles: Vec<DetailGroupRole> = vec![];
        for item in user_group_roles {
            let mut role: Option<Role> = None;
            if item.role_id.is_some() {
                role = match get_role_by_id(&mut tx, &item.role_id.unwrap()).await {
                    Ok(val) => val,
                    Err(err) => {
                        return UserDetailResponses::InternalServerError(Json(
                            InternalServerErrorResponse::new(
                                "route.user",
                                "user_detail_api",
                                "get role from user_group_roles",
                                &err.to_string(),
                            ),
                        ))
                    }
                };
            }
            let mut group: Option<Group> = None;
            if item.group_id.is_some() {
                group = match get_group_by_id(&mut tx, &item.group_id.unwrap()).await {
                    Ok(val) => val,
                    Err(err) => {
                        return UserDetailResponses::InternalServerError(Json(
                            InternalServerErrorResponse::new(
                                "route.user",
                                "user_detail_api",
                                "get group from user_role_groups",
                                &err.to_string(),
                            ),
                        ))
                    }
                };
            }
            group_roles.push(DetailGroupRole {
                role: role.map(|x| DetailRole {
                    id: x.id.to_string(),
                    role_name: x.role_name,
                }),
                group: group.map(|x| DetailGroup {
                    id: x.id.to_string(),
                    group_name: x.group_name,
                }),
            });
        }

        UserDetailResponses::Ok(Json(UserDetailResponse {
            id: user.id.to_string(),
            user_name: user.user_name,
            is_active: user.is_active,
            is_2faenabled: user.is_2faenabled,
            created_date: datetime_to_string_opt(user.created_date),
            updated_date: datetime_to_string_opt(user.updated_date),
            user_profile: user_profile.map(|x| DetailUserProfile {
                first_name: x.first_name,
                last_name: x.last_name,
                email: x.email,
                address: x.address,
            }),
            created_by: created_by.map(|x| DetailCreatedOrUpdatedUser {
                id: x.id.to_string(),
                user_name: x.user_name,
            }),
            updated_by: updated_by.map(|x| DetailCreatedOrUpdatedUser {
                id: x.id.to_string(),
                user_name: x.user_name,
            }),
            group_roles,
        }))
    }

    #[oai(path = "/user/", method = "post", tag = "ApiUserTags::User")]
    async fn user_create_api(
        &self,
        Json(json): Json<UserCreateRequest>,
        state: Data<&Arc<AppState>>,
        auth: BearerAuthorization,
    ) -> UserCreateResponses {
        // Begin db transaction
        let mut tx = match state.db.begin().await {
            Ok(val) => val,
            Err(err) => {
                return UserCreateResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.user",
                        "user_create_api",
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
                return UserCreateResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.user",
                        "user_create_api",
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
                    return UserCreateResponses::InternalServerError(Json(
                        InternalServerErrorResponse::new(
                            "route.user",
                            "user_create_api",
                            "get user from token",
                            &err.to_string(),
                        ),
                    ))
                }
            };
        if request_user.is_none() {
            return UserCreateResponses::Unauthorized(Json(UnauthorizedResponse::default()));
        }
        let now = Local::now().fixed_offset();
        // Insert User and User Profile
        let request_user = request_user.unwrap();
        let hashed_password = match hash_password(&json.password) {
            Ok(val) => val,
            Err(err) => {
                return UserCreateResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.user",
                        "user_create_api",
                        "hash_password",
                        &err.to_string(),
                    ),
                ));
            }
        };
        let new_user = User {
            id: Uuid::now_v7(),
            user_name: json.user_name,
            password: hashed_password,
            is_active: Some(json.is_active),
            is_2faenabled: Some(false),
            created_by: Some(request_user.id),
            updated_by: Some(request_user.id),
            created_date: Some(now),
            updated_date: Some(now),
            deleted_date: None,
        };
        let new_user_profile = UserProfile {
            id: Uuid::now_v7(),
            user_id: new_user.id,
            first_name: json.first_name,
            last_name: json.last_name,
            address: json.address,
            email: json.email,
        };
        if let Err(err) = create_user(&mut tx, &new_user, &new_user_profile).await {
            return UserCreateResponses::InternalServerError(Json(
                InternalServerErrorResponse::new(
                    "route.user",
                    "user_create_api",
                    "create_user",
                    &err.to_string(),
                ),
            ));
        }
        // Insert User Group Roles
        let mut user_group_roles: Vec<UserGroupRoles> = vec![];
        let mut group_roles_res: Vec<DetailGroupRole> = vec![];
        if json.group_roles.is_some() {
            let group_roles = json.group_roles.unwrap();
            for item in group_roles {
                let role_id = match Uuid::parse_str(&item.role_id) {
                    Ok(val) => val,
                    Err(_) => {
                        return UserCreateResponses::BadRequest(Json(BadRequestResponse {
                            message: format!("role with id = {} not found", &item.role_id),
                        }))
                    }
                };
                let role = match get_role_by_id(&mut tx, &role_id).await {
                    Ok(val) => val,
                    Err(err) => {
                        return UserCreateResponses::InternalServerError(Json(
                            InternalServerErrorResponse::new(
                                "route.user",
                                "user_create_api",
                                "check role",
                                &err.to_string(),
                            ),
                        ));
                    }
                };
                if role.is_none() {
                    return UserCreateResponses::BadRequest(Json(BadRequestResponse {
                        message: format!("role with id = {} not found", &item.role_id),
                    }));
                }
                let role = role.unwrap();
                let group_id = match Uuid::parse_str(&item.group_id) {
                    Ok(val) => val,
                    Err(_) => {
                        return UserCreateResponses::BadRequest(Json(BadRequestResponse {
                            message: format!("group with id = {} not found", &item.group_id),
                        }))
                    }
                };
                let group = match get_group_by_id(&mut tx, &group_id).await {
                    Ok(val) => val,
                    Err(err) => {
                        return UserCreateResponses::InternalServerError(Json(
                            InternalServerErrorResponse::new(
                                "route.user",
                                "user_create_api",
                                "check group",
                                &err.to_string(),
                            ),
                        ));
                    }
                };
                if group.is_none() {
                    return UserCreateResponses::BadRequest(Json(BadRequestResponse {
                        message: format!("group with id = {} not found", &item.group_id),
                    }));
                }
                let group = group.unwrap();
                user_group_roles.push(UserGroupRoles {
                    id: Uuid::now_v7(),
                    user_id: Some(new_user.id),
                    group_id: Some(group_id),
                    role_id: Some(role_id),
                });
                group_roles_res.push(DetailGroupRole {
                    role: Some(DetailRole {
                        id: role.id.to_string(),
                        role_name: role.role_name,
                    }),
                    group: Some(DetailGroup {
                        id: group.id.to_string(),
                        group_name: group.group_name,
                    }),
                });
            }
            if let Err(err) = upsert_user_group_roles(&mut tx, &new_user, &user_group_roles).await {
                return UserCreateResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.user",
                        "user_create_api",
                        "upsert_user_group_roles",
                        &err.to_string(),
                    ),
                ));
            }
        }

        if let Err(err) = tx.commit().await {
            return UserCreateResponses::InternalServerError(Json(
                InternalServerErrorResponse::new(
                    "route.user",
                    "user_create_api",
                    "commit to database",
                    &err.to_string(),
                ),
            ));
        }

        UserCreateResponses::Created(Json(UserCreateResponse {
            id: new_user.id.to_string(),
            user_name: new_user.user_name,
            is_active: new_user.is_active,
            group_roles: group_roles_res,
            user_profile: Some(DetailUserProfile {
                first_name: new_user_profile.first_name,
                last_name: new_user_profile.last_name,
                email: new_user_profile.email,
                address: new_user_profile.address,
            }),
        }))
    }

    #[oai(path = "/user/", method = "put", tag = "ApiUserTags::User")]
    async fn user_update_api(
        &self,
        Query(_id): Query<String>,
        Json(_json): Json<UserUpdateRequest>,
        _state: Data<&Arc<AppState>>,
        _auth: BearerAuthorization,
    ) -> UserUpdateResponses {
        todo!()
    }

    #[oai(path = "/user/", method = "delete", tag = "ApiUserTags::User")]
    async fn user_delete_api(
        &self,
        Query(_id): Query<String>,
        _state: Data<&Arc<AppState>>,
        _auth: BearerAuthorization,
    ) -> UserDeleteResponses {
        todo!()
    }

    #[oai(
        path = "/user/reset_passwd/",
        method = "post",
        tag = "ApiUserTags::User"
    )]
    async fn reset_password_api(
        &self,
        Json(_json): Json<ResetPasswordRequest>,
        _state: Data<&Arc<AppState>>,
        _auth: BearerAuthorization,
    ) -> ResetPasswordResponses {
        todo!()
    }

    #[oai(
        path = "/user/change-status/",
        method = "put",
        tag = "ApiUserTags::User"
    )]
    async fn change_status_api(
        &self,
        Query(_id): Query<String>,
        Json(_json): Json<ChangeStatusRequest>,
        _state: Data<&Arc<AppState>>,
        _auth: BearerAuthorization,
    ) -> ChangeStatusResponses {
        todo!()
    }

    #[oai(
        path = "/user/add-group-role/",
        method = "post",
        tag = "ApiUserTags::User"
    )]
    async fn add_user_group_role_api(
        &self,
        Json(_json): Json<AddUserGroupRoleRequest>,
        _state: Data<&Arc<AppState>>,
        _auth: BearerAuthorization,
    ) -> AddUserGroupRoleResponses {
        todo!()
    }

    #[oai(
        path = "/user/delete-group-role/",
        method = "post",
        tag = "ApiUserTags::User"
    )]
    async fn delete_user_group_role_api(
        &self,
        Query(_user_id): Query<String>,
        Query(_role_id): Query<String>,
        Query(_group_id): Query<String>,
        _state: Data<&Arc<AppState>>,
        _auth: BearerAuthorization,
    ) -> DeleteUserGroupRoleResponses {
        todo!()
    }
}
