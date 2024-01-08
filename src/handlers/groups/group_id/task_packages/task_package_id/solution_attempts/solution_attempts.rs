// Documentation was created by ChatGPT
use actix_web::{
    post,
    web::{self, Data, Json, Path, ServiceConfig},
    HttpResponse,
};

use crate::{
    jwt,
    models::{
        solution_attempts::{CreateSolutionAttemptSchema, CreatedSolutionAttemptResponse},
        task_package::TaskPackagePath,
    },
    permission,
    repository::group::GroupRepo,
};

/// # Create Solution Attempt
/// 
/// This route is used to create a new solution attempt within a group. 
/// It allows users to initiate a solution attempt for a specific task package. 
/// The created solution attempt includes metadata such as the user, task package, visibility, and initial state. 
/// Access is restricted based on user permissions.
#[utoipa::path(
    post,
    path="/api/groups/{group_id}/task_packages/{task_package_id}/solution_attempts/",
    tag="solution_attempt",
    params(
        ("task_package_id" = Uuid, Path, description = "The unique identifier for the task package associated with the solution attempt."),
        ("group_id" = Uuid, Path, description = "The unique identifier for the group associated with the task package."),
    ),
    request_body = CreateSolutionAttemptSchema,
    responses(
        (status = 201, description = "The solution attempt is successfully created", body = CreatedSolutionAttemptResponse),
    )
)]
#[post("/")]
pub async fn create_solution_attempt(
    body: Json<CreateSolutionAttemptSchema>,
    path: Path<TaskPackagePath>,
    data: Data<GroupRepo>,
    jwt: jwt::JwtMiddleware,
    _: permission::PermissionMiddleware,
) -> HttpResponse {
    //TODO Not Found if wrong task_package_id or group_id
    match data.create_solution_attempt(&jwt.user_id, &path.task_package_id, &path.group_id, &body.visibility) {
        Ok(created_soltuion_group) => {
            HttpResponse::Created().json(CreatedSolutionAttemptResponse {
                id: created_soltuion_group.solution_attempt.id,
                task_package_id: created_soltuion_group.solution_attempt.task_package_id,
                user_id: created_soltuion_group.solution_attempt.user_id,
                visibility: created_soltuion_group.solution_attempt.visibility,
                created_at: created_soltuion_group.solution_attempt.created_at.and_utc(),
                answer_list: created_soltuion_group.solution_list,
                state: created_soltuion_group.state,
            })
        }
        Err(_) => HttpResponse::InternalServerError()
            .json(serde_json::json!({"message": "Something went wrong"})),
    }
}

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(web::scope("/solution_attempts")
        .service(create_solution_attempt)
    );
}
