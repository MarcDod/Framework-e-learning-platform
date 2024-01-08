use utoipa::OpenApi;

use crate::handlers;
use crate::models::task::{
    CreateTaskRequest,
    CreateTaskResponse,
    TaskPagingResponse,
    TasksResponse,
    TaskResponse,
    TaskSolution,
    CreateSchemaRequest,
    SchemaPagingResponse,
    SchemaPagingSchema,
    RemoveTasksFromPackageResponse,
    RemoveTasksFromPackageSchema,
    TaskDoc
};

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::tasks::tasks::create_task,
        handlers::tasks::tasks::create_schema,
        handlers::tasks::tasks::fetch_schemas,
        handlers::tasks::task_id::task_id::fetch_task,
        handlers::tasks::task_id::task_id::delete_task,
        handlers::tasks::task_id::task_id::fetch_task_solution,
        handlers::tasks::tasks::fetch_tasks,
        handlers::groups::group_id::task_packages::task_package_id::tasks::tasks::add_tasks_to_package,
        //handlers::groups::group_id::task_packages::task_package_id::tasks::tasks::get_tasks_from_package,
        handlers::groups::group_id::task_packages::task_package_id::tasks::tasks::remove_tasks_from_package,
    ),
    components(schemas(
        CreateTaskRequest,
        CreateTaskResponse,
        TaskPagingResponse,
        TasksResponse,
        TaskResponse,
        TaskSolution,
        CreateSchemaRequest,
        SchemaPagingResponse,
        SchemaPagingSchema,
        RemoveTasksFromPackageResponse,
        RemoveTasksFromPackageSchema,
        TaskDoc
    )), 
    tags(
        (name="task", description = "task endpoints."),
        (name="task_doc", description = "Task dokuments.")
    ), 
)]
pub struct ApiDoc;