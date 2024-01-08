use utoipa::OpenApi;

use crate::handlers;
use crate::models::solution_attempts::{
    SolutionAttemptsResponse,
    SolutionAttemptResponse,
    CreateSolutionAttemptSchema,
    CreatedSolutionAttemptResponse,
    AnswerEntry,
    SolutionAttemptWithAnswerListResponse,
};

use crate::models::util::Visibility;

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::groups::group_id::task_packages::task_package_id::solution_attempts::solution_attempts::create_solution_attempt,
        handlers::groups::group_id::solution_attempts::solution_attempt_id::solution_attempt_id::finish_solution_attempt,
        handlers::groups::group_id::solution_attempts::solution_attempt_id::solution_attempt_id::fetch_solution_attempt,
        handlers::groups::group_id::task_packages::task_package_id::users::user_id::solution_attempts::solution_attempts::fetch_user_solution_attempts,
    ), 
    components(schemas(
        SolutionAttemptsResponse,
        SolutionAttemptResponse,
        CreateSolutionAttemptSchema,
        CreatedSolutionAttemptResponse,
        Visibility,
        AnswerEntry,
        SolutionAttemptWithAnswerListResponse,
    )), 
    tags(
        (name="solution_attempt", description = "Endpoints for accessing solution attempts"),
    ), 
)]
pub struct ApiDoc;