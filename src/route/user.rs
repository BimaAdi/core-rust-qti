use std::sync::Arc;

use chrono::Local;
use poem::web::Data;
use poem_openapi::{param::Query, payload::Json, OpenApi, Tags};
use uuid::Uuid;

use crate::{
    core::security::{get_user_from_token, hash_password, BearerAuthorization},
    model::{user::User, user_group_roles::UserGroupRoles, user_profile::UserProfile},
    repository::{
        group::get_group_by_id,
        role::get_role_by_id,
        user::{create_user, upsert_user_group_roles},
    },
    schema::{
        common::{BadRequestResponse, InternalServerErrorResponse, UnauthorizedResponse},
        user::{
            AddUserGroupRoleRequest, AddUserGroupRoleResponses, ChangeStatusRequest,
            ChangeStatusResponses, DeleteUserGroupRoleResponses, DetailGroup, DetailGroupRole,
            DetailRole, DetailUserProfile, GetAllUserResponses, GetPaginateUserResponses,
            ResetPasswordRequest, ResetPasswordResponses, UserCreateRequest, UserCreateResponse,
            UserCreateResponses, UserDeleteResponses, UserDetailResponses, UserUpdateRequest,
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
        Query(_page): Query<Option<u32>>,
        Query(_page_size): Query<Option<u32>>,
        Query(_search): Query<Option<String>>,
        _state: Data<&Arc<AppState>>,
        _auth: BearerAuthorization,
    ) -> GetPaginateUserResponses {
        todo!()
    }

    #[oai(path = "/user/all/", method = "get", tag = "ApiUserTags::User")]
    async fn get_all_user_api(
        &self,
        Query(_page): Query<Option<u32>>,
        Query(_page_size): Query<Option<u32>>,
        Query(_search): Query<Option<String>>,
        _state: Data<&Arc<AppState>>,
        _auth: BearerAuthorization,
    ) -> GetAllUserResponses {
        todo!()
    }

    #[oai(path = "/user/detail/", method = "get", tag = "ApiUserTags::User")]
    async fn user_detail_api(
        &self,
        Query(_id): Query<String>,
        _state: Data<&Arc<AppState>>,
        _auth: BearerAuthorization,
    ) -> UserDetailResponses {
        todo!()
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
                    role: DetailRole {
                        id: role.id.to_string(),
                        role_name: role.role_name,
                    },
                    group: DetailGroup {
                        id: group.id.to_string(),
                        group_name: group.group_name,
                    },
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
