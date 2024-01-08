// Documentation was created by ChatGPT
use std::io::{self, Write};

use actix_web::{web::{ServiceConfig, self, Data, Path, Json}, HttpResponse, get, patch};
use jsonschema::JSONSchema;
use serde_json::Value;
use uuid::Uuid;

use crate::{models::{answer::{AnswerPath, UpdatedAnswerResponse, UpdateAnswerSchema, AnswerResponse}, util::{AnswerState, AccessType}}, repository::{group::GroupRepo, postgres::PgRepo}, jwt, permission, AppState};

/// # Fetch Answer
///
/// This route is used for fetching details about a specific answer to a task. 
/// It returns information such as the answer's state, correctness, answer_doc, and timestamps. 
/// Users can only access details about their own answers unless they have the specified permission.
#[utoipa::path(
    get,
    path="/api/groups/{group_id}/answers/{answer_id}/",
    tag="answer",
    params(
        ("anser_id" = Uuid, Path, description = "The unique identifier for the answer being fetched."),
        ("group_id" = Uuid, Path, description = "The unique identifier of the group."),
    ),
    responses(
        (status = 200, description = "The request was successful, and the answer is provided.", body = AnswerResponse),
        (status = 403, description = "The user is not allowed to access the answer", body = ErrorSchema),
        (status = 404, description = "Answer could not be found", body = ErrorSchema),
    )
)]
#[get("/")]
pub async fn fetch_answer(
    path: Path<AnswerPath>,
    data: Data<GroupRepo>,
    app: Data<AppState>,
    jwt: jwt::JwtMiddleware,
    permission: permission::PermissionMiddleware,
) -> HttpResponse {
    let answer = match data.fetch_answer(&path.answer_id, &path.group_id) {
        Ok(solution) => solution,
        Err(_) => return HttpResponse::NotFound().json(
            serde_json::json!({"message": "Not Found"})
        )
    };

    if answer.created_from != jwt.user_id
        && !permission.permission_addons.contains(&AccessType::Other) {
        return HttpResponse::Forbidden().json(
            serde_json::json!({"message": "Not allowed"})
        )
    }

    let answer_doc = match app.mongodb.fetch_answer_doc(answer.answer_doc_id).await {
        Ok(Some(answer_doc)) => answer_doc,
        Ok(None) => return HttpResponse::Ok().json(AnswerResponse {
            id: answer.id,
            correct: answer.correct,
            state: answer.state,
            task_id: answer.task_id,
            answer_doc_id: answer.answer_doc_id,
            answer_doc: None,
            updated_at: None,
            created_from: answer.created_from,
        }),
        Err(_) => return HttpResponse::InternalServerError().json(
            serde_json::json!({"message": "Something went wrong"})
        )
    };

    HttpResponse::Ok().json(AnswerResponse {
        id: answer.id,
        state: answer.state,
        correct: answer.correct,
        task_id: answer.task_id,
        answer_doc_id: answer.answer_doc_id,
        answer_doc: Some(answer_doc.solution),
        updated_at: Some(answer_doc.updated_at.and_utc()),
        created_from: answer.created_from,
    })
}

/// # Update Answer Endpoint
///
/// This route is used for updating an answer to a task. 
/// The answer can only be updated by the user who created it. 
/// The updated answer must adhere to the schema defined for the corresponding task.
#[utoipa::path(
    patch,
    path="/api/groups/{group_id}/answers/{answer_id}/",
    tag="answer",
    request_body = UpdateAnswerSchema,
    params(
        ("answer_id" = Uuid, Path, description = "The unique identifier for the answer being updated."),
        ("group_id" = Uuid, Path, description = "The unique identifier of the group."),
    ),
    responses(
        (status = 200, description = "JSON object containing the updated answer details", body = UpdatedAnswerResponse),
        (status = 400, description = "The updated answer has the wrong format", body = ErrorSchema),
        (status = 401, description = "The user is not allowed to update the answer", body = ErrorSchema),
        (status = 403, description = "The answer is not in the 'Active' state", body = ErrorSchema),
        (status = 404, description = "The solution schema could not be found", body = ErrorSchema),
    )
)]
#[patch("/")]
pub async fn update_answer(
    body: Json<UpdateAnswerSchema>,
    path: Path<AnswerPath>,
    data: Data<GroupRepo>,
    app: Data<AppState>,
    jwt: jwt::JwtMiddleware,
    _: permission::PermissionMiddleware,
) -> HttpResponse {
    let answer = match data.fetch_answer(&path.answer_id, &path.group_id) {
        Ok(solution) => solution,
        Err(_) => return HttpResponse::InternalServerError().json(
            serde_json::json!({"message": "Something went wrong"})
        )
    };

    if answer.created_from != jwt.user_id {
        return HttpResponse::Forbidden().json(
            serde_json::json!({"message": "Not allowed"})
        )
    }

    if answer.state != AnswerState::Active {
        return HttpResponse::Forbidden().json(
            serde_json::json!({"message": "Soltuion is not Active"})
        );
    }

    let task_type = match data.fetch_task_type(&answer.task_id) {
        Ok(task_type) => task_type,
        Err(_) => return HttpResponse::InternalServerError().json(
            serde_json::json!({"message": "Something went wrong"})
        )
    };

    let schema_doc = match app.mongodb.fetch_schema(task_type.to_string()).await {
        Ok(Some(schema_doc)) => schema_doc,
        Ok(None) => return HttpResponse::NotFound().json(
            serde_json::json!({"message": "Schema could not be found"})
        ),
        Err(err) => return HttpResponse::InternalServerError().json(
            serde_json::json!({"message": "Something went wrong"})
        )
    };

    let solution_schema = match JSONSchema::compile(&serde_json::from_str(&serde_json::to_string(&schema_doc.solution_schema).unwrap()).unwrap()) {
        Ok(v) => v,
        Err(_) => return HttpResponse::InternalServerError().json(
            serde_json::json!({"message": "Schema could not be parsed"})
        )
    };

    let solution_string = body.solution.to_string();
    let solution_obj: Value = match serde_json::from_str(&solution_string) {
        Ok(v) => v,
        Err(_) => return HttpResponse::InternalServerError().json(
            serde_json::json!({"message": "solution could not be parsed"})
        )
    };

    match solution_schema.validate(&solution_obj) {
        Ok(_) => (),
        Err(_) => return HttpResponse::BadRequest().json(
            serde_json::json!({"message": "solution has wrong format"})
        )
    }

    let updated_answer_doc; 

    if answer.answer_doc_id == Uuid::default() {
        updated_answer_doc = match app.mongodb.create_answer_doc(body.solution.clone()).await {
            Ok(created_solution) => created_solution,
            Err(_) => return HttpResponse::InternalServerError().json(
                serde_json::json!({"message": "Something went wrong"})
            )
        };

        let amounts: usize = match data.insert_answer_doc(&updated_answer_doc.id, &answer.id) {
            Ok(v) => v,
            Err(_) => {
                app.mongodb.delete_solution(&updated_answer_doc.id).await;

                return HttpResponse::InternalServerError().json(
                    serde_json::json!({"message": "Something went wrong"})
                )
            }
        };
    } else {
        updated_answer_doc = match app.mongodb.update_answer(body.solution.clone(), answer.answer_doc_id).await {
            Ok(updated_solution) => updated_solution,
            Err(_) => return HttpResponse::InternalServerError().json(
                serde_json::json!({"message": "Something went wrong"})
            )
        }
    }

    HttpResponse::Ok().json(UpdatedAnswerResponse {
        id: answer.id,
        task_id: answer.task_id,
        answer_doc_id: updated_answer_doc.id,
        solution: updated_answer_doc.solution,
        updated_at: updated_answer_doc.updated_at.and_utc(),
        created_from: answer.created_from,
    })
}

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(
        web::scope("/{answer_id}")
            .service(update_answer)
            .service(fetch_answer)
    );
}