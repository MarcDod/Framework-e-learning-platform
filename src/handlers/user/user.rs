// Documentation was created by ChatGPT
use actix_web::{web::{ServiceConfig, self, Data, Query}, get, HttpResponse};

use crate::{repository::{postgres::PgRepo, users::UsersRepo, permissions::PermissionsRepo}, models::{users::UserResponse, util::{OrderDir, PagingSchema}, permissions::{PermissionRequest, PermissionListResponse}}, permission, jwt};

use super::groups::groups;

/// # Get Current User Endpoint
///
/// This endpoint allows fetching information about the currently authenticated user.
#[utoipa::path(
    get,
    path = "/api/user/",
    tag = "user",
    responses(
        (status = 200, description = "Successfully retrieved information about the current user.", body = UserResponse),
    )
)]
#[get("/")]
pub async fn me(
    data: Data<UsersRepo>,
    jwt: jwt::JwtMiddleware,
    _: permission::PermissionMiddleware,
) -> HttpResponse {
    let user = data.fetch_user_by_id(jwt.user_id);
    
    match user {
        Ok(user) => HttpResponse::Ok().json(UserResponse {
            id: user.id,
            email: user.email,
            name: user.name
        }),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}

/// # Get My Global Permissions Endpoint
///
/// This endpoint allows fetching a paginated list of global permissions for the authenticated user.
#[utoipa::path(
    get,
    path = "/api/user/permissions",
    tag = "user",
    params(
        ("ressources[]" = Option<String>, Query, description = "A list of resource names to filter permissions."),
        ("page" = Option<i32>, Query, description = "The page number for paginated results (default: 0)."),
        ("limit" = Option<i32>, Query, description = "The number of permissions to retrieve per page (default: 200)."),
        ("order" = Option<OrderDir>, Query, description = "The order of the results (default: DESC)."),
    ),
    responses(
        (status = 200, description = "Successfully retrieved the paginated list of global permissions.", body = PermissionListResponse),
    )
)]
#[get("/permissions")]
pub async fn get_my_global_permissions(
    data: Data<PermissionsRepo>,
    query: Query<PermissionRequest>,
    jwt: jwt::JwtMiddleware,
    _: permission::PermissionMiddleware,
) -> HttpResponse {
    let pagination = &PagingSchema{
        limit: query.limit.unwrap_or(200),
        page: query.page.unwrap_or(0),
        order: query.order.unwrap_or(OrderDir::DESC),
    };

    let gloabl_permission_list = 
        match data.fetch_user_permissions(&jwt.user_id, pagination, &None, true, &query.ressources) {
            Ok(v) => v,
            Err(_) => return HttpResponse::InternalServerError().json(
                serde_json::json!({"message": "Something went wrong"})
            )
        }; 

    HttpResponse::Ok().json(PermissionListResponse {
        permission_list: gloabl_permission_list.permission_list.into_iter()
        .map(|permission| permission.clone()).collect(),
    })
}

pub fn config(cfg: &mut ServiceConfig, pgdb: PgRepo) {
    let user_repo = pgdb.new_user_repo();
    cfg.service(
        web::scope("/user")
            .app_data(Data::<UsersRepo>::new(user_repo.clone()))
            .service(me)
            .service(get_my_global_permissions)
            .configure(|cfg| groups::config(cfg, pgdb.clone()))
    );
}