use std::sync::Arc;

use chrono::Local;
use poem::web::Data;
use poem_openapi::{param::Query, payload::Json, OpenApi, Tags};
use uuid::Uuid;

use crate::{
    core::security::{get_user_from_token, BearerAuthorization},
    model::permission_attribute::PermissionAttribute,
    repository::permission_attribute::{
        create_permission_attribute, delete_permission_attribute, get_all_permission_attribute,
        get_permission_attribute_by_id, update_permission_attribute,
    },
    schema::{
        common::{
            InternalServerErrorResponse, NotFoundResponse, PaginateResponse, UnauthorizedResponse,
        },
        permission_attribute::{
            CreatePermissionAttributeRequest, CreatePermissionAttributeResponses,
            DeletePermissionAttributeResponses, DetailPermissionAttribute,
            DetailPermissionAttributeResponses, DropdownPermissionAttributeResponses,
            PaginatePermissionAttributeResponses, UpdatePermissionAttributeRequest,
            UpdatePermissionAttributeResponses,
        },
    },
    AppState,
};

#[derive(Tags)]
enum ApiPermissionAttributeTags {
    PermissionAttribute,
}

pub struct ApiPermissionAttribute;

#[OpenApi]
impl ApiPermissionAttribute {
    #[oai(
        path = "/permission-attribute/",
        method = "get",
        tag = "ApiPermissionAttributeTags::PermissionAttribute"
    )]
    async fn paginate_permission_attribute_api(
        &self,
        Query(page): Query<Option<u32>>,
        Query(page_size): Query<Option<u32>>,
        Query(search): Query<Option<String>>,
        state: Data<&Arc<AppState>>,
        auth: BearerAuthorization,
    ) -> PaginatePermissionAttributeResponses {
        // Begin db transaction
        let mut tx = match state.db.begin().await {
            Ok(val) => val,
            Err(err) => {
                return PaginatePermissionAttributeResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.permission",
                        "paginate_permission_attribute_api",
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
                return PaginatePermissionAttributeResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.permission",
                        "paginate_permission_attribute_api",
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
                return PaginatePermissionAttributeResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.permission",
                        "paginate_permission_attribute_api",
                        "get user from token",
                        &err.to_string(),
                    ),
                ))
            }
        };
        if user.is_none() {
            return PaginatePermissionAttributeResponses::Unauthorized(Json(
                UnauthorizedResponse::default(),
            ));
        }
        let page = page.unwrap_or(1);
        let page_size = page_size.unwrap_or(10);
        let (data, counts, page_count) = match get_all_permission_attribute(
            &mut tx,
            Some(page),
            Some(page_size),
            search,
            None,
            None,
        )
        .await
        {
            Ok(val) => val,
            Err(err) => {
                return PaginatePermissionAttributeResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.permission",
                        "paginate_permission_api",
                        "get_all_permission",
                        &err.to_string(),
                    ),
                ))
            }
        };
        PaginatePermissionAttributeResponses::Ok(Json(PaginateResponse {
            counts,
            page,
            page_count,
            page_size,
            results: data
                .iter()
                .map(|x| DetailPermissionAttribute {
                    id: x.id.to_string(),
                    name: x.name.clone(),
                    description: x.description.clone(),
                })
                .collect(),
        }))
    }

    #[oai(
        path = "/permission-attribute/dropdown/",
        method = "get",
        tag = "ApiPermissionAttributeTags::PermissionAttribute"
    )]
    async fn dropdown_permission_attribute_api(
        &self,
        Query(limit): Query<Option<u32>>,
        state: Data<&Arc<AppState>>,
        auth: BearerAuthorization,
    ) -> DropdownPermissionAttributeResponses {
        // Begin db transaction
        let mut tx = match state.db.begin().await {
            Ok(val) => val,
            Err(err) => {
                return DropdownPermissionAttributeResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.permission",
                        "dropdown_permission_attribute_api",
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
                return DropdownPermissionAttributeResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.permission",
                        "dropdown_permission_attribute_api",
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
                return DropdownPermissionAttributeResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.permission",
                        "dropdown_permission_attribute_api",
                        "get user from token",
                        &err.to_string(),
                    ),
                ))
            }
        };
        if user.is_none() {
            return DropdownPermissionAttributeResponses::Unauthorized(Json(
                UnauthorizedResponse::default(),
            ));
        }

        let (data, _, _) = match get_all_permission_attribute(
            &mut tx,
            None,
            None,
            None,
            limit,
            Some(true),
        )
        .await
        {
            Ok(val) => val,
            Err(err) => {
                return DropdownPermissionAttributeResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.permission",
                        "dropdown_permission_attribute_api",
                        "get_all_permission",
                        &err.to_string(),
                    ),
                ))
            }
        };

        DropdownPermissionAttributeResponses::Ok(Json(
            data.iter()
                .map(|x| DetailPermissionAttribute {
                    id: x.id.to_string(),
                    name: x.name.clone(),
                    description: x.description.clone(),
                })
                .collect(),
        ))
    }

    #[oai(
        path = "/permission-attribute/detail/",
        method = "get",
        tag = "ApiPermissionAttributeTags::PermissionAttribute"
    )]
    async fn detail_permission_attribute_api(
        &self,
        Query(id): Query<String>,
        state: Data<&Arc<AppState>>,
        auth: BearerAuthorization,
    ) -> DetailPermissionAttributeResponses {
        // Begin db transaction
        let mut tx = match state.db.begin().await {
            Ok(val) => val,
            Err(err) => {
                return DetailPermissionAttributeResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.permission_attribute",
                        "detail_permission_attribute_api",
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
                return DetailPermissionAttributeResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.permission_attribute",
                        "detail_permission_attribute_api",
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
                return DetailPermissionAttributeResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.permission_attribute",
                        "detail_permission_attribute_api",
                        "get user from token",
                        &err.to_string(),
                    ),
                ))
            }
        };
        if user.is_none() {
            return DetailPermissionAttributeResponses::Unauthorized(Json(
                UnauthorizedResponse::default(),
            ));
        }
        let id = match Uuid::parse_str(&id) {
            Ok(val) => val,
            Err(_) => {
                return DetailPermissionAttributeResponses::NotFound(Json(NotFoundResponse {
                    message: format!("permission_attribute_id with id = {} not found", id),
                }))
            }
        };
        let data = match get_permission_attribute_by_id(&mut tx, &id).await {
            Ok(val) => val,
            Err(err) => {
                return DetailPermissionAttributeResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.permission_attribute",
                        "detail_permission_attribute_api",
                        "get user from token",
                        &err.to_string(),
                    ),
                ))
            }
        };
        if data.is_none() {
            return DetailPermissionAttributeResponses::NotFound(Json(NotFoundResponse {
                message: format!("permission_attribute_id with id = {} not found", id),
            }));
        }
        let data = data.unwrap();
        DetailPermissionAttributeResponses::Ok(Json(DetailPermissionAttribute {
            id: data.id.to_string(),
            name: data.name,
            description: data.description,
        }))
    }

    #[oai(
        path = "/permission-attribute/",
        method = "post",
        tag = "ApiPermissionAttributeTags::PermissionAttribute"
    )]
    async fn create_permission_attribute_api(
        &self,
        Json(json): Json<CreatePermissionAttributeRequest>,
        state: Data<&Arc<AppState>>,
        auth: BearerAuthorization,
    ) -> CreatePermissionAttributeResponses {
        // Begin db transaction
        let mut tx = match state.db.begin().await {
            Ok(val) => val,
            Err(err) => {
                return CreatePermissionAttributeResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.permission_attribute",
                        "create_permission_attribute_api",
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
                return CreatePermissionAttributeResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.permission_attribute",
                        "create_permission_attribute_api",
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
                return CreatePermissionAttributeResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.permission_attribute",
                        "create_permission_attribute_api",
                        "get user from token",
                        &err.to_string(),
                    ),
                ))
            }
        };
        if user.is_none() {
            return CreatePermissionAttributeResponses::Unauthorized(Json(
                UnauthorizedResponse::default(),
            ));
        }
        let now = Local::now().fixed_offset();
        let new_permission = PermissionAttribute {
            id: Uuid::now_v7(),
            name: json.name,
            description: json.description,
            created_date: Some(now),
            updated_date: Some(now),
        };
        if let Err(err) = create_permission_attribute(&mut tx, &new_permission).await {
            return CreatePermissionAttributeResponses::InternalServerError(Json(
                InternalServerErrorResponse::new(
                    "route.permission_attribute",
                    "create_permission_attribute_api",
                    "create_permission_attribute",
                    &err.to_string(),
                ),
            ));
        }
        if let Err(err) = tx.commit().await {
            return CreatePermissionAttributeResponses::InternalServerError(Json(
                InternalServerErrorResponse::new(
                    "route.permission_attribute",
                    "create_permission_attribute_api",
                    "commit transaction",
                    &err.to_string(),
                ),
            ));
        }
        CreatePermissionAttributeResponses::Ok(Json(DetailPermissionAttribute {
            id: new_permission.id.to_string(),
            name: new_permission.name,
            description: new_permission.description,
        }))
    }

    #[oai(
        path = "/permission-attribute/",
        method = "put",
        tag = "ApiPermissionAttributeTags::PermissionAttribute"
    )]
    async fn update_permission_attribute_api(
        &self,
        Query(id): Query<String>,
        Json(json): Json<UpdatePermissionAttributeRequest>,
        state: Data<&Arc<AppState>>,
        auth: BearerAuthorization,
    ) -> UpdatePermissionAttributeResponses {
        // Begin db transaction
        let mut tx = match state.db.begin().await {
            Ok(val) => val,
            Err(err) => {
                return UpdatePermissionAttributeResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.permission_attribute",
                        "update_permission_attribute_api",
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
                return UpdatePermissionAttributeResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.permission_attribute",
                        "update_permission_attribute_api",
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
                return UpdatePermissionAttributeResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.permission_attribute",
                        "update_permission_attribute_api",
                        "get user from token",
                        &err.to_string(),
                    ),
                ))
            }
        };
        if user.is_none() {
            return UpdatePermissionAttributeResponses::Unauthorized(Json(
                UnauthorizedResponse::default(),
            ));
        }
        let id = match Uuid::parse_str(&id) {
            Ok(val) => val,
            Err(_) => {
                return UpdatePermissionAttributeResponses::NotFound(Json(NotFoundResponse {
                    message: format!("permission_attribute_id with id = {} not found", id),
                }))
            }
        };
        let data = match get_permission_attribute_by_id(&mut tx, &id).await {
            Ok(val) => val,
            Err(err) => {
                return UpdatePermissionAttributeResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.permission_attribute",
                        "update_permission_attribute_api",
                        "get user from token",
                        &err.to_string(),
                    ),
                ))
            }
        };
        if data.is_none() {
            return UpdatePermissionAttributeResponses::NotFound(Json(NotFoundResponse {
                message: format!("permission_attribute_id with id = {} not found", id),
            }));
        }
        let mut data = data.unwrap();
        let now = Local::now().fixed_offset();
        data.name = json.name;
        data.description = json.description;
        data.updated_date = Some(now);
        if let Err(err) = update_permission_attribute(&mut tx, &data).await {
            return UpdatePermissionAttributeResponses::InternalServerError(Json(
                InternalServerErrorResponse::new(
                    "route.permission_attribute",
                    "update_permission_attribute_api",
                    "update_permission_attribute",
                    &err.to_string(),
                ),
            ));
        }
        if let Err(err) = tx.commit().await {
            return UpdatePermissionAttributeResponses::InternalServerError(Json(
                InternalServerErrorResponse::new(
                    "route.permission_attribute",
                    "update_permission_attribute_api",
                    "commit transaction",
                    &err.to_string(),
                ),
            ));
        }
        UpdatePermissionAttributeResponses::Ok(Json(DetailPermissionAttribute {
            id: data.id.to_string(),
            name: data.name,
            description: data.description,
        }))
    }

    #[oai(
        path = "/permission-attribute/",
        method = "delete",
        tag = "ApiPermissionAttributeTags::PermissionAttribute"
    )]
    async fn delete_permission_attribute_api(
        &self,
        Query(id): Query<String>,
        state: Data<&Arc<AppState>>,
        auth: BearerAuthorization,
    ) -> DeletePermissionAttributeResponses {
        // Begin db transaction
        let mut tx = match state.db.begin().await {
            Ok(val) => val,
            Err(err) => {
                return DeletePermissionAttributeResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.permission_attribute",
                        "delete_permission_attribute_api",
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
                return DeletePermissionAttributeResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.permission_attribute",
                        "delete_permission_attribute_api",
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
                return DeletePermissionAttributeResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.permission_attribute",
                        "delete_permission_attribute_api",
                        "get user from token",
                        &err.to_string(),
                    ),
                ))
            }
        };
        if user.is_none() {
            return DeletePermissionAttributeResponses::Unauthorized(Json(
                UnauthorizedResponse::default(),
            ));
        }
        let id = match Uuid::parse_str(&id) {
            Ok(val) => val,
            Err(_) => {
                return DeletePermissionAttributeResponses::NotFound(Json(NotFoundResponse {
                    message: format!("permission_attribute_id with id = {} not found", id),
                }))
            }
        };
        let data = match get_permission_attribute_by_id(&mut tx, &id).await {
            Ok(val) => val,
            Err(err) => {
                return DeletePermissionAttributeResponses::InternalServerError(Json(
                    InternalServerErrorResponse::new(
                        "route.permission_attribute",
                        "delete_permission_attribute_api",
                        "get user from token",
                        &err.to_string(),
                    ),
                ))
            }
        };
        if data.is_none() {
            return DeletePermissionAttributeResponses::NotFound(Json(NotFoundResponse {
                message: format!("permission_attribute_id with id = {} not found", id),
            }));
        }
        let data = data.unwrap();
        if let Err(err) = delete_permission_attribute(&mut tx, &data).await {
            return DeletePermissionAttributeResponses::InternalServerError(Json(
                InternalServerErrorResponse::new(
                    "route.permission_attribute",
                    "delete_permission_attribute_api",
                    "delete_permission_attribute",
                    &err.to_string(),
                ),
            ));
        }
        if let Err(err) = tx.commit().await {
            return DeletePermissionAttributeResponses::InternalServerError(Json(
                InternalServerErrorResponse::new(
                    "route.permission_attribute",
                    "delete_permission_attribute_api",
                    "commit transaction",
                    &err.to_string(),
                ),
            ));
        }
        DeletePermissionAttributeResponses::NoContent
    }
}
