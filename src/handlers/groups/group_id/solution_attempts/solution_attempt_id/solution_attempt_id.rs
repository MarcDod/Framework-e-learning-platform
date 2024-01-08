// Documentation was created by ChatGPT
use std::collections::HashMap;

use actix_web::{web::{ServiceConfig, self, Path, Data, Query}, post, get, HttpResponse};
use uuid::Uuid;

use crate::{models::{solution_attempts::SolutionAttemptGroupPath, util}, AppState};
use crate::models::util::AccessType;
use crate::{models::{solution_attempts::{SolutionAttemptPath, SolutionAttemptWithAnswerListResponse}, util::Visibility, users::{UserPath, UserQuery}}, permission, jwt, repository::{group::GroupRepo, postgres::PgRepo}};

/// # Finish Solution Attempt
/// 
/// This route is used to finish a solution attempt for a specific group. 
/// It validates and compares the submitted solutions with the expected solutions for each task in the attempt and updates the state accordingly. 
/// Only users with the necessary permissions or the solution attempt owner can finish the attempt.
/// 
/// Only users with the necessary permissions or the solution attempt owner can finish the attempt.
#[utoipa::path(
    post,
    path="/api/groups/{group_id}/solution_attempts/{solution_attempt_id}/finish",
    tag="solution_attempt",
    params(
        ("solution_attempt_id" = Uuid, Path, description = "The unique identifier for the solution attempt to be finished."),
        ("group_id" = Uuid, Path, description = "The unique identifier for the group associated with the solution attempt."),
    ),
    responses(
        (status = 204, description = "The solution attempt is successfully finished"),
        (status = 400, description = "The solution attempt is already finished"),
    )
)]
#[post("/finish")]
pub async fn finish_solution_attempt(
    path: Path<SolutionAttemptGroupPath>,
    data: Data<GroupRepo>,
    app: Data<AppState>,
    jwt: jwt::JwtMiddleware,
    permission: permission::PermissionMiddleware,
) -> HttpResponse {
    let solution_attempt = match data.fetch_solution_attempt(&path.solution_attempt_id, &path.group_id) {
        Ok(solution_attempt) => solution_attempt,
        Err(_) => return HttpResponse::InternalServerError()
            .json(serde_json::json!({"message": "Something went wrong"})),
    };

    if solution_attempt.solution_attempt.visibility == Visibility::Private
        && solution_attempt.solution_attempt.user_id != jwt.user_id {
        return  HttpResponse::Forbidden().json(serde_json::json!({"message": "Forbidden access to Solution group"}));
    }

    if solution_attempt.solution_attempt.user_id != jwt.user_id
        && !permission.permission_addons.contains(&AccessType::Other) {
        return  HttpResponse::Forbidden().json(serde_json::json!({"message": "Forbidden access to Solution group"}));
    }

    let mut answer_accurate_state: HashMap<Uuid, bool> = HashMap::new();

    for answer in solution_attempt.solution_list {
        let answer_doc = match app.mongodb.fetch_answer_doc(answer.answer_doc_id).await {
            Ok(answer) => answer,
            Err(_) => return HttpResponse::InternalServerError().json(
                serde_json::json!({"message": "Something went wrong"})
            )
        };

        if answer_doc.is_none() {
            answer_accurate_state.insert(answer.answer_id, false);
            continue;
        }
        let task_doc = match app.mongodb.fetch_task(answer.task_doc_id, false).await {
            Ok(task_doc) => task_doc.unwrap(),
            Err(_) => return HttpResponse::InternalServerError().json(
                serde_json::json!({"message": "Something went wrong"})
            )
        };

        answer_accurate_state.insert(answer.answer_id, util::compare_json_objects_ignore_order(
            &serde_json::from_str(&answer_doc.unwrap().solution.to_string()).unwrap(), 
            &serde_json::from_str(&task_doc.solution.to_string()).unwrap()
        ));
    }

    let amount_updated_solutions = match data.finish_solution_attempt(&path.solution_attempt_id, &answer_accurate_state) {
        Ok(amount_updated_solutions) => amount_updated_solutions,
        Err(_) => return HttpResponse::InternalServerError().json(
            serde_json::json!({"message": "Something went wrong"})
        )
    };

    if amount_updated_solutions == 0 {
        return HttpResponse::BadRequest().json(
            serde_json::json!({"message": "Solution Group already finished"})
        );
    }

    HttpResponse::NoContent().finish()
}


/// # Fetch Solution Attempt
/// 
/// This route is used to fetch details of a specific solution attempt within a group. 
/// It retrieves information about the solutions submitted in the attempt, along with metadata about the attempt. 
/// Access is restricted based on the visibility of the solution attempt and user permissions.
/// 
/// The route provides details about the solutions submitted in the attempt, along with metadata about the attempt itself.
/// Access is restricted based on the visibility of the solution attempt and user permissions.
#[utoipa::path(
    get,
    path="/api/groups/{group_id}/solution_attempts/{solution_attempt_id}/",
    tag="solution_attempt",
    params(
        ("solution_attempt_id" = Uuid, Path, description = "The unique identifier for the solution attempt to be fetched."),
        ("group_id" = Uuid, Path, description = "The unique identifier for the group associated with the solution attempt."),
    ),
    responses(
        (status = 200, description = "The solution attempt is successfully fetched.", body = SolutionAttemptWithAnswerListResponse),
        (status = 403, description = "The user is not allowed to access the solution attempt")
    )
)]
#[get("/")]
pub async fn fetch_solution_attempt(
    path: Path<SolutionAttemptGroupPath>,
    data: Data<GroupRepo>,
    jwt: jwt::JwtMiddleware,
    permission: permission::PermissionMiddleware,
) -> HttpResponse {
    //TODO: not found on wrong solution_attempt_id or group_id
    let solution_attempt = match data.fetch_solution_attempt(&path.solution_attempt_id, &path.group_id) {
        Ok(solution_group) => solution_group,
        Err(_) => return HttpResponse::InternalServerError()
            .json(serde_json::json!({"message": "Something went wrong"})),
    };

    if solution_attempt.solution_attempt.visibility == Visibility::Private 
        && solution_attempt.solution_attempt.user_id != jwt.user_id {
        return  HttpResponse::Forbidden().json(serde_json::json!({"message": "Forbidden access to Solution group"}));
    }

    if solution_attempt.solution_attempt.user_id != jwt.user_id
        && !permission.permission_addons.contains(&AccessType::Other) {
        return  HttpResponse::Forbidden().json(serde_json::json!({"message": "Forbidden access to Solution group"}));
    }

    HttpResponse::Ok().json(SolutionAttemptWithAnswerListResponse {
        id: solution_attempt.solution_attempt.id,
        task_package_id: solution_attempt.solution_attempt.task_package_id,
        user_id: solution_attempt.solution_attempt.user_id,
        visibility: solution_attempt.solution_attempt.visibility,
        created_at: solution_attempt.solution_attempt.created_at.and_utc(),
        answer_list: solution_attempt.solution_list,
        state: solution_attempt.state,
    })
}


pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(
        web::scope("/{solution_attempt_id}")
            .service(fetch_solution_attempt)
            .service(finish_solution_attempt)
    );
}