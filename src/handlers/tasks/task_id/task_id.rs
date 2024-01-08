// Documentation was created by ChatGPT
use std::io::{self, Write};

use actix_web::{web::{ServiceConfig, self, Data, Json, Path}, HttpResponse, post, get, delete};
use jsonschema::{JSONSchema, Draft};
use serde_json::Value;

use crate::{jwt, permission, AppState, models::task::{CreateTaskRequest, CreateTaskResponse, TaskPath, TaskResponse, TaskSolution}};

// # Delete Task Endpoint
///
/// This endpoint deletes an existing task based on the provided task ID.
#[utoipa::path(
    delete,
    path = "/api/tasks/{task_id}/",
    tag = "task_doc",
    params(
        ("task_id" = Uuid, Path, description = "The unique identifier of the task to be deleted."),
    ),
    responses(
        (status = 204, description = "The task was successfully deleted."),
    ),
)]
#[delete("/")]
pub async fn delete_task(
    path: Path<TaskPath>,
    app: Data<AppState>,
    _: jwt::JwtMiddleware,
    _: permission::PermissionMiddleware,
) -> HttpResponse {
    match app.mongodb.delete_task(&path.task_id).await {
        Ok(_) => return HttpResponse::NoContent().finish(),
        Err(_) => return HttpResponse::InternalServerError().json(
            serde_json::json!({"message": "Something went wrong"})
        )
    }
}

/// # Fetch Task Endpoint
///
/// This endpoint retrieves details of a specific task based on the provided task ID.
#[utoipa::path(
    get,
    path = "/api/tasks/{task_id}/",
    tag = "task_doc",
    params(
        ("task_id" = Uuid, Path, description = "The unique identifier of the task to be fetched."),
    ),
    responses(
        (status = 200, description = "The request was successful, and task details are provided.", body = TaskResponse),
        (status = 404, description = "The specified task ID does not exist.")
    ),
)]
#[get("/")]
pub async fn fetch_task(
    path: Path<TaskPath>,
    app: Data<AppState>,
    _: jwt::JwtMiddleware,
    _: permission::PermissionMiddleware,
) -> HttpResponse {

    let task = match app.mongodb.fetch_task(path.task_id, false).await {
        Ok(v) => v,
        Err(_) => return HttpResponse::InternalServerError().json(
            serde_json::json!({"message": "Something went wrong"})
        )
    };

    if let Some(task) = task {
        return HttpResponse::Ok().json(TaskResponse {
            id: task.id,
            state: task.state,
            task: task.task,
            task_type: task.task_type,
        });
    } else {
        return HttpResponse::NotFound().json(
            serde_json::json!({"message": "Task not found"})
        )
    }
}

/// # Fetch Task Solution Endpoint
///
/// This endpoint retrieves the solution details of a specific task based on the provided task ID.
#[utoipa::path(
    get,
    path = "/api/tasks/{task_id}/solution",
    tag = "task_doc",
    params(
        ("task_id" = Uuid, Path, description = "The unique identifier of the task for which the solution is to be fetched."),
    ),
    responses(
        (status = 200, description = "The request was successful, and task details are provided.", body = TaskSolution),
        (status = 404, description = "The task with the specified identifier was not found.")
    ),
)]
#[get("/solution")]
pub async fn fetch_task_solution(
    path: Path<TaskPath>,
    app: Data<AppState>,
    _: jwt::JwtMiddleware,
    _: permission::PermissionMiddleware,
) -> HttpResponse {

    let task = match app.mongodb.fetch_task(path.task_id, false).await {
        Ok(v) => v,
        Err(_) => return HttpResponse::InternalServerError().json(
            serde_json::json!({"message": "Something went wrong"})
        )
    };

    if let Some(task) = task {
        return HttpResponse::Ok().json(TaskSolution {
            solution: task.solution
        });
    } else {
        return HttpResponse::NotFound().json(
            serde_json::json!({"message": "Task not found"})
        )
    }
}

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(
        web::scope("/{task_id}")
            .service(fetch_task)
            .service(delete_task)
            .service(fetch_task_solution)
    );
}