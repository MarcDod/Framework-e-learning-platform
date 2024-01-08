// Documentation was created by ChatGPT
use actix_web::{web::{ServiceConfig, self, Path, Query, Data}, get, HttpResponse, HttpRequest};

use crate::{models::{util::{OrderDir, PagingSchema}, groups::GroupPath, permissions::{PermissionRequest, PermissionListResponseWithCount}}, permission, jwt, repository::permissions::PermissionsRepo};

/// # Get Group Permissions Endpoint
///
/// This endpoint retrieves the permissions associated with a specific group for a given user.
#[utoipa::path(
    get,
    path = "/api/groups/{group_id}/user/permissions",
    tag = "group",
    params(
        ("group_id" = Uuid, Path, description = "The unique identifier of the group for which permissions are being fetched."),
        ("ressources[]" = Option<String>, Query, description = "The values that should be looked for"),
        ("page" = Option<i32>, Query, description = "The page number for pagination (default: 0)."),
        ("limit" = Option<i32>, Query, description = "The maximum number of permissions to retrieve per page (default: 200)."),
        ("order" = Option<OrderDir>, Query, description = "The order of the results (default: DESC).")
    ),
    responses(
        (status = 200, description = "The request was successful, and a list of permissions is provided.", body = PermissionListResponse),
    )
)]
#[get("/permissions")]
pub async fn get_group_permissions(
    data: Data<PermissionsRepo>,
    query: Query<PermissionRequest>,
    path: Path<GroupPath>,
    jwt: jwt::JwtMiddleware,
    _: permission::PermissionMiddleware,
) -> HttpResponse {
    let pagination = &PagingSchema{
        limit: query.limit.unwrap_or(200),
        page: query.page.unwrap_or(0),
        order: query.order.unwrap_or(OrderDir::DESC),
    };

    let permission_list = 
        match data.fetch_user_permissions(&jwt.user_id, pagination, &Some(path.group_id), true, &query.ressources) {
            Ok(v) => v,
            Err(_) => return HttpResponse::InternalServerError().json(
                serde_json::json!({"message": "Something went wrong"})
            )
        };

    HttpResponse::Ok().json(PermissionListResponseWithCount {
        permission_list: permission_list.permission_list,
        total_count: permission_list.total_count,
    })
}

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(
        web::scope("/user")
            .service(get_group_permissions)
    );
}