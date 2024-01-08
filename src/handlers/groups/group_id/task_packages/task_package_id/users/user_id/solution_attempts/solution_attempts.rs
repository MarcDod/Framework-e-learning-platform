// Documentation was created by ChatGPT
use actix_web::{
    get,
    web::{self, Data, Path, ServiceConfig, Query},
    HttpResponse,
};

use crate::{
    jwt,
    models::{
        solution_attempts::{SolutionAttemptResponse, SolutionAttemptsResponse, VisibilityQuery},
        task_package::{TaskPackagePath, TaskPackageUserPath}, util::{Visibility, AccessType}, users::UserQuery,
    },
    permission,
    repository::group::GroupRepo, schema::solution_attempts::visibility,
};

/// # Fetch Public Solution Attempts Endpoint
///
/// This endpoint retrieves public solution attempts for a specific task package and user within a group.
///
#[utoipa::path(
    get,
    path="/api/groups/{group_id}/task_packages/{task_package_id}/users/{user_id}/solution_attempts/",
    tag="solution_attempt",
    params(
        ("task_package_id" = Uuid, Path, description = "The unique identifier of the task package for which public solution attempts are being fetched."),
        ("group_id" = Uuid, Path, description = "The unique identifier of the group."),
        ("user_id" = Uuid, Path, description = "The unique identifier of the user."),
        ("visibility" = Option<Visibility>, Query, description = "The visibility of requested solution attempts")
    ),
    responses(
        (status = 200, description = "The request was successful, and solution attempts are provided.", body = SolutionAttemptsResponse),
        (status = 493, description = "The requester does not have permission to fetch solution attempts.", body = ErrorSchema),
    )
)]
#[get("/")]
pub async fn fetch_user_solution_attempts(
    path: Path<TaskPackageUserPath>,
    data: Data<GroupRepo>,
    query: Query<VisibilityQuery>,
    jwt: jwt::JwtMiddleware,
    permission: permission::PermissionMiddleware,
) -> HttpResponse {
    if jwt.user_id != path.user_id
        && (!permission.permission_addons.contains(&AccessType::Other) || query.visibility == Some(Visibility::Private)) {
            return HttpResponse::Forbidden().json(
                serde_json::json!({"message": "No permission"})
            );
        }
    let mut visi: &Option<Visibility> = &query.visibility.clone();

    if visi.is_none() && jwt.user_id != path.user_id {
       visi = &Some(Visibility::Public)
    }

    // TODO: paging
    let solution_attempts = match data.fetch_solution_attempts(&path.user_id, &path.task_package_id, &path.group_id, visi) {
        Ok(solution_groups) => solution_groups,
        Err(_) => return HttpResponse::InternalServerError()
            .json(serde_json::json!({"message": "Something went wrong"})),
    };

    HttpResponse::Ok().json(SolutionAttemptsResponse {
        solution_attempts: solution_attempts
            .into_iter()
            .map(|solution_group| SolutionAttemptResponse {
                id: solution_group.0.id,
                task_package_id: solution_group.0.task_package_id,
                user_id: solution_group.0.user_id,
                visibility: solution_group.0.visibility,
                created_at: solution_group.0.created_at.and_utc(),
                state: solution_group.1,
            })
            .collect(),
    })
}

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(
        web::scope("/solution_attempts")
            .service(fetch_user_solution_attempts)
    );
}