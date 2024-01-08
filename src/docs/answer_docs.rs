use utoipa::OpenApi;

use crate::handlers;
use crate::models::answer::{
    AnswersResponse, AnswerResponse,
    UpdateAnswerSchema, UpdatedAnswerResponse,
};

use crate::models::util::AnswerState;

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::groups::group_id::answers::answer_id::answer_id::fetch_answer,
        handlers::groups::group_id::answers::answer_id::answer_id::update_answer,
    ), 
    components(schemas(
        AnswersResponse,
        AnswerResponse,
        UpdateAnswerSchema,
        UpdatedAnswerResponse,
        AnswerState,
    )), 
    tags(
        (name="answer", description = "These endpoints provide options to view and edit user-submitted responses."),
    ), 
)]
pub struct ApiDoc;