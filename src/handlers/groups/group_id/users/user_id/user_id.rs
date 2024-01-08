// Documentation was created by ChatGPT
use std::{collections::HashMap, io::{self, Write}};

use actix_web::{web::{ServiceConfig, self, Data, Path, Json, Query}, get, post, HttpResponse};

use crate::{models::{groups::{AddPermissionResponse, NewUserPermission, GroupUserPath, AddPermissionSchema}, permissions::{PermissionRequest, RessourceListWithCount, PermissionInfo, PermissionListResponse, PermissionListResponseWithCount}, util::{PagingSchema, OrderDir, AccessType}}, permission, jwt, repository::{permissions::PermissionsRepo, group::GroupRepo}};

/// # Get Group Permissions from User Endpoint
///
/// This endpoint retrieves the permissions associated with a specific group for a given user.
#[utoipa::path(
    get,
    path = "/api/groups/{group_id}/users/{user_id}/permissions",
    tag = "group",
    params(
        ("group_id" = Uuid, Path, description = "The unique identifier of the group for which permissions are being fetched."),
        ("user_id" = Uuid, Path, description = "The unique identifier of the user for whom permissions are being fetched."),
        ("ressources[]" = Option<String>, Query, description = "List of all ressources that should be looked for"),
        ("page" = Option<i32>, Query, description = "The page number for pagination (default: 0)."),
        ("limit" = Option<i32>, Query, description = "The maximum number of permissions to retrieve per page (default: 200)."),
        ("order" = Option<OrderDir>, Query, description = "The order of results (ascending or descending). Default is set to descending (DESC)."),
        ("group_only" = Option<bool>, Query, description = "If set to true, only group-specific permissions are retrieved.")
    ),    
    responses(
        (status = 200, description = "The request was successful, and a list of permissions is provided.", body = PermissionListResponseWithCount),
        (status = 403, description = "The user does not have permission to access the requested information.", body = PermissionListResponseWithCount),
    ),
)]
#[get("/permissions")]
pub async fn get_group_permissions_from_user(
    data: Data<PermissionsRepo>,
    path: Path<GroupUserPath>,
    query: Query<PermissionRequest>,
    jwt: jwt::JwtMiddleware,
    permission: permission::PermissionMiddleware,
) -> HttpResponse {    
    let pagination = &PagingSchema{
        limit: query.limit.unwrap_or(200),
        page: query.page.unwrap_or(0),
        order: query.order.unwrap_or(OrderDir::DESC),
    };

    if jwt.user_id != path.user_id
        && !permission.permission_addons.contains(&AccessType::Other) {
        return HttpResponse::Forbidden().json(
            serde_json::json!({"message": "No permission"})
        );
    }

    let user_permission_list = match data.fetch_user_permissions(
        &path.user_id, 
        pagination, 
        &Some(path.group_id),
        query.group_only,
        &query.ressources,
    ) {
        Ok(v) => v,
        Err(_) => return HttpResponse::InternalServerError().json(
            serde_json::json!({"message": "Something went wrong"})
        )
    };


    HttpResponse::Ok().json(PermissionListResponseWithCount {
        permission_list: user_permission_list.permission_list,
        total_count: user_permission_list.total_count,
    })
}

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(
        web::scope("/{user_id}")
            .service(get_group_permissions_from_user)
    );
}