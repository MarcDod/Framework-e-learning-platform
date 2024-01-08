// Documentation was created by ChatGPT
use actix_web::{web::{ServiceConfig, self, Data, Path, Query}, HttpResponse, get};

use crate::{models::{task_package::{TaskPackageUserPath, TaskPackageUserStatisticResponse}, util::{AccessType, Visibility}, task::TaskTypeFilter}, repository::group::GroupRepo, permission, jwt, schema::solution_attempts::visibility};

use super::solution_attempts::solution_attempts;

/// # Fetch Task Package Statistic Endpoint
///
/// This endpoint retrieves statistics for a specific task package and user within a group, allowing users to analyze their performance and completion rates across different task types.
///
#[utoipa::path(
    get,
    path="/api/groups/{group_id}/task_packages/{task_package_id}/users/{user_id}/statistic",
    tag="task_package",
    params(
        ("task_package_id" = Uuid, Path, description = "The unique identifier of the task package for which statistics are being retrieved."),
        ("group_id" = Uuid, Path, description = "The unique identifier of the group to which the user belongs."),
        ("user_id" = Uuid, Path, description = "The unique identifier of the user whose statistics are being fetched."),
        ("task_types[]" = Option<String>, Query, description = "Filter the statistics by specific task types.")
    ),
    responses(
        (status = 200, description = "The request was successful, and task package statistics are provided.", body = TaskPackageUserStatisticResponse),
        (status = 403, description = "The requester does not have permission to fetch task package statistics.", body = ErrorSchema),
    )
)]
#[get("/statistic")]
pub async fn fetch_task_package_statistic(
    path: Path<TaskPackageUserPath>,
    data: Data<GroupRepo>,
    query: Query<TaskTypeFilter>,
    jwt: jwt::JwtMiddleware,
    permission: permission::PermissionMiddleware,
) -> HttpResponse {
    if jwt.user_id != path.user_id
        && !permission.permission_addons.contains(&AccessType::Other) {
        return HttpResponse::Forbidden().json(
            serde_json::json!({"message": "No permission"})
        );
    }
    let tasks = match data.fetch_tasks_from_package(&path.task_package_id, &path.group_id, &query.task_types) {
        Ok(v) => v,
        Err(_) => return HttpResponse::InternalServerError().json(
            serde_json::json!({"message": "Something went wrong"})
        )
    };

    let statistics = 
        match data.fetch_task_package_user_statistic(
            &path.group_id, 
            &path.user_id, 
            &path.task_package_id, 
            &if jwt.user_id == path.user_id { None } else { Some(Visibility::Private) },
            &query.task_types
        ) {
            Ok(v) => v,
            Err(_) => return HttpResponse::InternalServerError().json(
                serde_json::json!({"message": "Something went wrong"})
            )
        };
    
    HttpResponse::Ok().json(
        TaskPackageUserStatisticResponse {
            amount_tasks: tasks.len(),
            values: statistics,
        }
    )
}

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(
        web::scope("/{user_id}")
            .service(fetch_task_package_statistic)
            .configure(solution_attempts::config)
    );
}