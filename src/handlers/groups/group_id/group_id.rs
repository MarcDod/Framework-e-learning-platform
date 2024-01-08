// Documentation was created by ChatGPT
use actix_web::{web::{ServiceConfig, self, Path, Data}, HttpResponse, delete, get};

use crate::{repository::{postgres::PgRepo, group::GroupRepo}, permission, jwt, models::groups::{GroupPath, GroupInfoResponse, GroupMetaDataResponse}};

use super::{users::users, members::members, task_packages::task_packages, answers::anwers, solution_attempts::solution_attempts, user::user};


/// # Get Group Information Endpoint
///
/// This endpoint retrieves information about a specific group.
#[utoipa::path(
    get,
    path = "/api/groups/{group_id}/",
    tag = "group",
    params(
        ("group_id" = Uuid, Path, description = "The unique identifier of the group for which information is being fetched."),
    ),
    responses(
        (status = 200, description = "The request was successful, and information about the group is provided.", body = GroupInfoResponse),
        (status = 404, description = "Group could not be found", body = ErrorSchema),
    )
)]
#[get("/")]
pub async fn get_group(
    data: Data<GroupRepo>,
    path: Path<GroupPath>,
    _: jwt::JwtMiddleware,
    _: permission::PermissionMiddleware,
) -> HttpResponse {
    let group_info = match data.fetch_active_group_info(path.group_id) {
        Ok(v) => v,
        Err(_) => return HttpResponse::NotFound().json(
            serde_json::json!({"message": "Something went wrong"})
        )
    };

    HttpResponse::Ok().json(GroupInfoResponse {
        id: group_info.id,
        name: group_info.name,
        parent: group_info.parent,
    })
}

/// # Get Group Metadata Endpoint
///
/// This endpoint retrieves metadata information about a specific group.
#[utoipa::path(
    get,
    path = "/api/groups/{group_id}/metadata",
    tag = "group",
    params(
        ("group_id" = Uuid, Path, description = "The unique identifier of the group for which metadata is being fetched."),
    ),
    responses(
        (status = 200, description = "The request was successful, and metadata about the group is provided.", body = GroupMetaDataResponse),
        (status = 404, description = "Group could not be found", body = ErrorSchema),
    )
)]
#[get("/metadata")]
pub async fn get_group_meta_data(
    data: Data<GroupRepo>,
    path: Path<GroupPath>,
    _: jwt::JwtMiddleware,
    _: permission::PermissionMiddleware,
) -> HttpResponse {
    let group_meta_data = match data.fetch_active_group_meta_data(path.group_id) {
        Ok(v) => v,
        Err(_) => return HttpResponse::NotFound().json(
            serde_json::json!({"message": "Group couldnt be found"})
        )
    };

    HttpResponse::Ok().json(GroupMetaDataResponse {
        id: group_meta_data.id,
        created_from: group_meta_data.created_from,
        created_at: group_meta_data.created_at.and_utc(),
        updated_from: group_meta_data.updated_from,
        updated_at: group_meta_data.updated_at.and_utc(),
    })
}

/// # Delete Group Endpoint
///
/// This endpoint deletes a specific group.
#[utoipa::path(
    delete,
    path = "/api/groups/{group_id}",
    tag = "group",
    params(
        ("group_id" = Uuid, Path, description = "The unique identifier of the group to be deleted."),
    ),
    responses(
        (status = 204, description = "The group was successfully deleted."),
    )
)]
#[delete("/")]
pub async fn delete_group(
    path: Path<GroupPath>,
    group_repo: Data<GroupRepo>,
    token_payload: jwt::JwtMiddleware,
    _: permission::PermissionMiddleware
) -> HttpResponse {
    match group_repo.delete_group(&path.group_id, &token_payload.user_id) {
        Ok(v) => {
            if v == 1 {
                return HttpResponse::NoContent().finish()
            }
            return HttpResponse::InternalServerError().json(
                serde_json::json!({"message": "Something went wrong"})
            )
        }
        Err(_) => return HttpResponse::InternalServerError().json(
            serde_json::json!({"message": "Something went wrong"})
        )
    }
}

pub fn config(cfg: &mut ServiceConfig, pgdb: PgRepo) {
    cfg.service(
        web::scope("/{group_id}")
            .service(get_group)
            .service(get_group_meta_data)
            .service(delete_group)
            .configure(task_packages::config)
            .configure(anwers::config)
            .configure(solution_attempts::config)
            .configure(user::config)
            .configure(users::config)
            .configure(|cfg| members::config(cfg, pgdb.clone()))
    );
}