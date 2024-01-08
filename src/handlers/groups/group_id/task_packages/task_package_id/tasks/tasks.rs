// Documentation was created by ChatGPT
use actix_web::{
    delete, get, post,
    web::{self, Data, Json, Path, Query, ServiceConfig},
    HttpResponse,
};
use uuid::Uuid;

use crate::{
    jwt,
    models::{
        task::{
            AddTasksToPackageSchema, AddedTasksToPackageResponse, TasksFromPackageResponse,
            NewTempTask, RemoveTasksFromPackageSchema, RemoveTasksFromPackageResponse,
            TaskTypeFilter, TasksResponse, TaskResponse,
        },
        task_package::TaskPackagePath,
        util::{OrderDir, PagingSchema},
    },
    permission,
    repository::group::GroupRepo,
    AppState,
};

/// # Add Tasks to a Task Package.
///
/// This route is used to add tasks to a task package within a group. 
/// It allows users to add multiple tasks to an existing task package. 
/// The tasks to be added are specified by their unique task document IDs. 
/// Access is restricted based on user permissions.
#[utoipa::path(
    post,
    path="/api/groups/{group_id}/task_packages/{task_package_id}/tasks/",
    tag="task",
    request_body = AddTasksToPackageSchema,
    params(
        ("task_package_id" = Uuid, Path, description = "The unique identifier for the task package where tasks will be added."),
        ("group_id" = Uuid, Path, description = "The unique identifier for the group associated with the task package."),
    ),
    responses(
        (status = 200, description = "The request was successful, and tasks are successfully added", body = AddedTasksToPackageResponse),
    )
)]
#[post("/")]
pub async fn add_tasks_to_package(
    body: Json<AddTasksToPackageSchema>,
    path: Path<TaskPackagePath>,
    data: Data<GroupRepo>,
    app: Data<AppState>,
    _: jwt::JwtMiddleware,
    _: permission::PermissionMiddleware,
) -> HttpResponse {
    let pagination = &PagingSchema {
        limit: 10,
        page: 0,
        order: OrderDir::DESC,
    };
    let tasks = match app
        .mongodb
        .fetch_all_tasks(pagination, Some(body.task_doc_ids.clone()), true)
        .await
    {
        Ok(v) => v,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"message": "Something went wrong"}))
        }
    };

    match data.add_tasks_to_package(
        &path.task_package_id,
        &path.group_id,
        &tasks
            .into_iter()
            .map(|task| NewTempTask {
                task_doc_id: task.id,
                task_type: task.task_type,
            })
            .collect::<Vec<NewTempTask>>(),
    ) {
        Ok(group_tasks) => {
            return HttpResponse::Ok().json(AddedTasksToPackageResponse {
                added_tasks: group_tasks,
            })
        }
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"message": "Something went wrong"}))
        }
    }
}

/// # Get Task Information from Task Package
///
/// This route is used to retrieve information about tasks within a specific task package in a group. 
/// It allows users to filter tasks based on their types. 
/// Access is restricted based on user permissions.
#[utoipa::path(
    get,
    path="/api/groups/{group_id}/task_packages/{task_package_id}/tasks/",
    tag="task_package",
    params(
        ("task_package_id" = Uuid, Path, description = "The unique identifier for the task package from which to retrieve task information."),
        ("group_id" = Uuid, Path, description = "The unique identifier for the group associated with the task package."),
        ("task_types[]" = Option<String>, Query, description = "Optional parameter to filter tasks by their types. If not provided, all tasks in the package are retrieved."),
    ),
    responses(
        (status = 200, description = "The request was successful, and task information is successfully retrieved", body = TasksFromPackageResponse),
    ),
)]
#[get("/")]
pub async fn get_tasks_infos_from_package(
    path: Path<TaskPackagePath>,
    data: Data<GroupRepo>,
    query: Query<TaskTypeFilter>,
    _: jwt::JwtMiddleware,
    _: permission::PermissionMiddleware,
) -> HttpResponse {
    match data.fetch_tasks_from_package(&path.task_package_id, &path.group_id, &query.task_types) {
        Ok(collection_tasks) => {
            return HttpResponse::Ok().json(TasksFromPackageResponse {
                tasks: collection_tasks,
            })
        }
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"message": "Something went wrong"}))
        }
    }
}

/// # Remove Tasks from Task Package
///
/// This route is used to remove tasks from a specific task package in a group. 
/// It allows users to specify the tasks to be removed. 
/// Access is restricted based on user permissions.
#[utoipa::path(
    delete,
    path="/api/groups/{group_id}/task_packages/{task_package_id}/tasks/",
    tag="task",
    params(
        ("task_package_id" = Uuid, Path, description = "The unique identifier for the task package from which to remove tasks."),
        ("group_id" = Uuid, Path, description = "The unique identifier for the group associated with the task package."),
    ),
    responses(
        (status = 200, description = "The request was successful, and tasks are successfully removed", body = RemoveTasksFromPackageResponse),
    ),
    request_body = RemoveTasksFromPackageSchema
)]
#[delete("/")]
pub async fn remove_tasks_from_package(
    body: Json<RemoveTasksFromPackageSchema>,
    path: Path<TaskPackagePath>,
    data: Data<GroupRepo>,
    _: jwt::JwtMiddleware,
    _: permission::PermissionMiddleware,
) -> HttpResponse {
    match data.remove_tasks_from_package(&path.task_package_id, &path.group_id, &body.task_ids) {
        Ok(tasks) => {
            return HttpResponse::Ok().json(RemoveTasksFromPackageResponse {
                removed_tasks: tasks,
            })
        }
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"message": "Something went wrong"}))
        }
    }
}

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(
        web::scope("/tasks")
            //.service(get_tasks_from_package)
            .service(remove_tasks_from_package)
            .service(add_tasks_to_package)
            .service(get_tasks_infos_from_package),
    );
}
