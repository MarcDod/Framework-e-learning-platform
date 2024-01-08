use utoipa::OpenApi;

use crate::handlers;
use crate::models::task_package::{
    CreateTaskPackageSchema, CreatedTaskPackageResponse,
    TaskPackagesResponse, TaskPackage,
    TaskPackageUserStatisticResponse,
    TaskPackageUserStatisticValue,
};

use crate::models::task::{
    AddTasksToPackageSchema, AddedTasksToPackageResponse,
    Task, TasksFromPackageResponse,
    RemoveTasksFromPackageSchema
};

use crate::models::util::TaskPackageType;

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::groups::group_id::task_packages::task_packages::create_task_package,
        handlers::groups::group_id::task_packages::task_packages::fetch_task_packages,
        handlers::groups::group_id::task_packages::task_package_id::users::user_id::user_id::fetch_task_package_statistic,
        handlers::groups::group_id::task_packages::task_package_id::tasks::tasks::get_tasks_infos_from_package,
    ), 
    components(schemas(
        CreateTaskPackageSchema,
        CreatedTaskPackageResponse,
        TaskPackagesResponse,
        TaskPackage,
        AddTasksToPackageSchema,
        AddedTasksToPackageResponse,
        Task,
        TasksFromPackageResponse,
        RemoveTasksFromPackageSchema,
        TaskPackageType,
        TaskPackageUserStatisticResponse,
        TaskPackageUserStatisticValue,
    )), 
    tags(
        (name="task_package", description = "task package endpoints."),
    ), 
)]
pub struct ApiDoc;