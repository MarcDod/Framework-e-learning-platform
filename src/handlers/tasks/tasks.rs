// Documentation was created by ChatGPT
use std::io::{self, Write};

use actix_web::{web::{ServiceConfig, Data, Query, self, Json}, HttpResponse, get, post};
use jsonschema::JSONSchema;
use serde_json::Value;

use crate::{AppState, jwt, permission, models::{task::{TaskPagingSchema, TaskPagingResponse, TaskResponse, CreateSchemaRequest, SchemaPagingSchema, SchemaPagingResponse, CreateTaskRequest, CreateTaskResponse}, util::{PagingSchema, OrderDir}}};

use super::task_id::task_id;

/// # Fetch Tasks Endpoint
///
/// This endpoint retrieves a paginated list of tasks based on the provided query parameters.
#[utoipa::path(
    get,
    path = "/api/tasks/",
    tag = "task_doc",
    params(
        ("task_ids[]" = Option<String>, Query, description = "A list of specific task identifiers to filter the results."),
        ("page" = Option<i32>, Query, description = "The page number of results if the request is paginated. Default is set to page 0."),
        ("limit" = Option<i32>, Query, description = "The maximum number of tasks to be returned per page."),
        ("order" = Option<OrderDir>, Query, description = "The order of results (ascending or descending). Default is set to descending (DESC)."),
    ),
    responses(
        (status = 200, description = "The request was successful, and tasks are provided.", body = TaskPagingResponse),
    ),
)]
#[get("/")]
pub async fn fetch_tasks(
    query: Query<TaskPagingSchema>,
    app: Data<AppState>,
    _: jwt::JwtMiddleware,
    _: permission::PermissionMiddleware,
) -> HttpResponse {
    let pagination = &PagingSchema{
        limit: query.limit.unwrap_or(200),
        page: query.page.unwrap_or(0),
        order: query.order.unwrap_or(OrderDir::DESC),
    };

    let tasks = match app.mongodb.fetch_all_tasks(pagination, query.task_ids.clone(), false).await {
        Ok(v) => v,
        Err(err) => {
            eprintln!("error: {err}");
            io::stdout().flush().unwrap();
            return HttpResponse::InternalServerError().json(
                serde_json::json!({"message": "Something went wrong"})
            )
        }
    };

    let total_count = match app.mongodb.fetch_amount_tasks(false).await {
        Ok(v) => v,
        Err(_) => return HttpResponse::InternalServerError().json(
            serde_json::json!({"message": "Something wen wrong"})
        )
    };

    HttpResponse::Ok().json(TaskPagingResponse {
        tasks: tasks.into_iter().map(|task| TaskResponse {
            id: task.id,
            task_type: task.task_type,
            state: task.state,
            task: task.task,
        }).collect(),
        total_count
    })
}

/// # Create Schema Endpoint
///
/// This endpoint allows the creation of a new schema for a specific task type.
///
#[utoipa::path(
    post,
    path = "/api/tasks/schemas",
    tag = "task_doc",
    request_body = CreateSchemaRequest,
    responses(
        (status = 204, description = "The schema was successfully created."),
        (status = 409, description = "A schema for the specified task type already exists."),
    ),
)]
#[post("/schemas")]
pub async fn create_schema(
    app: Data<AppState>,
    body: Json<CreateSchemaRequest>,
    _: jwt::JwtMiddleware,
    _: permission::PermissionMiddleware,
) -> HttpResponse {

    match app.mongodb.fetch_schema(body.task_type.to_string()).await {
        Ok(Some(_)) => return HttpResponse::Conflict().json(
            serde_json::json!({"message": "Tasktype already exists"})
        ),
        Ok(None) => (),
        Err(_) => return HttpResponse::InternalServerError().json(
            serde_json::json!({"message": "Something went wrong"})
        )
    };

    match JSONSchema::compile(&serde_json::from_str(&serde_json::to_string(&body.task_schema.clone()).unwrap()).unwrap()) {
        Ok(v) => v,
        Err(_) => return HttpResponse::BadRequest().json(
            serde_json::json!({"message": "Task schema could not be parsed"})
        )
    };

    match JSONSchema::compile(&serde_json::from_str(&serde_json::to_string(&body.solution_schema.clone()).unwrap()).unwrap()) {
        Ok(v) => v,
        Err(_) => return HttpResponse::BadRequest().json(
            serde_json::json!({"message": "Solution schema could not be parsed"})
        )
    };

    match app.mongodb.create_schema_doc(body.task_schema.clone(), body.solution_schema.clone(), body.task_type.to_string()).await {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(_) => HttpResponse::InternalServerError().json(
            serde_json::json!({"message": "Something wen wrong"})
        )
    }    
}

/// # Fetch Schemas Endpoint
///
/// This endpoint allows fetching a paginated list of schemas based on specified criteria.
#[utoipa::path(
    get,
    path = "/api/tasks/schemas",
    tag = "task_doc",
    params(
        ("task_types[]" = Option<String>, Query, description = "A list of specific task identifiers to filter the results."),
        ("page" = Option<i32>, Query, description = "The page number for paginated results (default: 0)."),
        ("limit" = Option<i32>, Query, description = "The number of schemas to retrieve per page (default: 200)."),
        ("order" = Option<OrderDir>, Query, description = "A list of task types to filter schemas."),
    ),
    responses(
        (status = 200, description = "Successfully retrieved the paginated list of schemas."),
    ),
)]
#[get("/schemas")]
pub async fn fetch_schemas(
    app: Data<AppState>,
    query: Query<SchemaPagingSchema>,
    _: jwt::JwtMiddleware,
    _: permission::PermissionMiddleware,
) -> HttpResponse {
    let pagination = &PagingSchema{
        limit: query.limit.unwrap_or(200),
        page: query.page.unwrap_or(0),
        order: query.order.unwrap_or(OrderDir::DESC),
    };
    let schemas = match app.mongodb.fetch_all_schemas(pagination, query.task_types.clone()).await {
        Ok(schemas) => schemas,
        Err(_) => return HttpResponse::InternalServerError().json(
            serde_json::json!({"message": "Something wen wrong"})
        )
    };

    let total_count = match app.mongodb.fetch_amount_schemas().await {
        Ok(v) => v,
        Err(_) => return HttpResponse::InternalServerError().json(
            serde_json::json!({"message": "Something wen wrong"})
        )
    };

    HttpResponse::Ok().json(
        SchemaPagingResponse {
            schemas,
            total_count,
        }
    )
}

/// # Create Task Endpoint
///
/// This endpoint creates a new task with the provided task type, task, and solution.
#[utoipa::path(
    post,
    path = "/api/tasks/",
    tag = "task_doc",
    request_body = CreateTaskRequest,
    responses(
        (status = 200, description = "The task was successfully created, and information about the new task is provided.", body = CreateTaskResponse),
        (status = 400, description = "The provided task or solution has an incorrect format.", body = ErrorSchema),
        (status = 401, description = "The user is not authorized to create tasks.", body = ErrorSchema),
    ),
)]
#[post("/")]
pub async fn create_task(
    app: Data<AppState>,
    body: Json<CreateTaskRequest>,
    _: jwt::JwtMiddleware,
    _: permission::PermissionMiddleware,
) -> HttpResponse {

    
    let schema_doc = match app.mongodb.fetch_schema(body.task_type.clone()).await {
        Ok(Some(schema_doc)) => schema_doc,
        Ok(None) => return HttpResponse::BadRequest().json(
            serde_json::json!({"message": "Schema could not be found"})
        ),
        Err(err) => return HttpResponse::InternalServerError().json(
            serde_json::json!({"message": "Something went wrong"})
        )
    };

    let task_schema = match JSONSchema::compile(&serde_json::from_str(&serde_json::to_string(&schema_doc.task_schema).unwrap()).unwrap()) {
        Ok(v) => v,
        Err(_) => return HttpResponse::InternalServerError().json(
            serde_json::json!({"message": "Schema could not be parsed"})
        )
    };

    let solution_schema = match JSONSchema::compile(&serde_json::from_str(&serde_json::to_string(&schema_doc.solution_schema).unwrap()).unwrap()) {
        Ok(v) => v,
        Err(_) => return HttpResponse::InternalServerError().json(
            serde_json::json!({"message": "Schema could not be parsed"})
        )
    };

    let task_string = body.task.to_string();
    let task_obj: Value = match serde_json::from_str(&task_string) {
        Ok(v) => v,
        Err(_) => return HttpResponse::InternalServerError().json(
            serde_json::json!({"message": "task could not be parsed"})
        )
    };

    match task_schema.validate(&task_obj) {
        Ok(_) => (),
        Err(_) => return HttpResponse::BadRequest().json(
            serde_json::json!({"message": "task has wrong format"})
        )
    }

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

    let created_task = match app.mongodb.create_task(body.task_type.to_string(), body.task.clone(), body.solution.clone()).await {
        Ok(created_task) => created_task,
        Err(_) => return HttpResponse::InternalServerError().json(
            serde_json::json!({"message": "Something went wrong"})
        )
    };

    HttpResponse::Ok().json(CreateTaskResponse {
        task: created_task
    })
}


pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(
        web::scope("/tasks")
            .service(fetch_tasks)
            .service(create_schema)
            .service(fetch_schemas)
            .service(create_task)
            .configure(task_id::config)
    );
}