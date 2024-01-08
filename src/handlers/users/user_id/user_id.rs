// Documentation was created by ChatGPT
use std::io::{self, Write};

use actix_web::{web::{ServiceConfig, self, Path, Data, Json, Query}, get, post, HttpResponse};

use crate::{models::{users::{UserResponse, UserPath}, groups::{AddPermissionSchema, AddPermissionResponse, NewUserPermission, GroupQuery}}, permission, jwt, repository::{users::UsersRepo, group::GroupRepo, permissions::PermissionsRepo}};

/// # Fetch User Info Endpoint
///
/// This endpoint allows fetching information about a specific user.
#[utoipa::path(
    get,
    path = "/api/users/{user_id}/info",
    tag = "user",
    responses(
        (status = 200, description = "User informations", body = UserResponse),
        (status = 404, description = "User not found", body = ErrorSchema),
    ),
    params(
        ("user_id" = Uuid, Path, description = "The unique identifier of the user for whom information is being retrieved."),
    )
)]
#[get("/info")]
pub async fn fetch_user_info(
    data: Data<UsersRepo>,
    path: Path<UserPath>,
    _: jwt::JwtMiddleware,
    _: permission::PermissionMiddleware,
) -> HttpResponse {
    let user = data.fetch_user_by_id(path.user_id);
    
    match user {
        Ok(user) => HttpResponse::Ok().json(UserResponse {
            id: user.id,
            email: user.email,
            name: user.name
        }),
        Err(err) => HttpResponse::NotFound().json(
            serde_json::json!({"message": "User not found"})
        ),
    }
}

/// # Add Permissions to User Endpoint
///
/// This endpoint allows adding permissions to a user without a specific group.
#[utoipa::path(
    post,
    path = "/api/users/{user_id}/permissions",
    tag = "group",
    request_body = AddPermissionSchema,
    params(
        ("group_id" = Option<Uuid>, Query, description = "The unique identifier of the group to which permissions are being added."),
        ("user_id" = Uuid, Path, description = "The unique identifier of the user to whom permissions are being added.")
    ),    
    responses(
        (status = 201, description = "The permissions were successfully added to the user. Returns a list of updated permissions.", body = AddPermissionResponse),
    ),
)]
#[post("/permissions")]
pub async fn add_permissions_to_user(
    body: Json<AddPermissionSchema>,
    path: Path<UserPath>,
    query: Query<GroupQuery>,
    group_repo: Data<GroupRepo>,
    permission_repo: Data<PermissionsRepo>,
    jwt: jwt::JwtMiddleware,
    _: permission::PermissionMiddleware,
) -> HttpResponse {

    let mut updated_permissions: Vec<String> = vec![];

    for permission in body.new_permissions.to_owned() {

        let new_group_permission = NewUserPermission{
            ressource: permission.value.to_string(),
            user_id: path.user_id,
            group_id: query.group_id,
        };

        if permission_repo.user_can_set_permissions(&new_group_permission, &permission.permission_addons, &jwt.user_id).unwrap_or(false) {
            let successfully_set_permission = group_repo.user_set_permission(&new_group_permission, &permission.permission_addons).unwrap_or(0);

            if successfully_set_permission > 0 {
                updated_permissions.push(permission.value);
            };
        }
    };

    HttpResponse::Created().json(AddPermissionResponse {
        updated_permissions
    })
}

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(
        web::scope("/{user_id}")
            .service(add_permissions_to_user)
            .service(fetch_user_info)
    );
}