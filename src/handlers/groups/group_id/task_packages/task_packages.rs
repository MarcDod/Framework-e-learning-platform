// Documentation was created by ChatGPT
use std::io::{self, Write};

use actix_web::{web::{ServiceConfig, Data, Path, self, Json}, post, HttpResponse, get};
use uuid::Uuid;

use crate::{models::{task_package::{TaskPackagesResponse, CreateTaskPackageSchema, CreatedTaskPackageResponse, CreatedTaskPackage}, groups::GroupPath, util::{PagingSchema, OrderDir}, task::NewTempTask}, permission, jwt, repository::group::GroupRepo, AppState};

use super::task_package_id::task_package_id;

/// # Fetch Task Packages Endpoint
/// 
/// This endpoint retrieves a list of task packages within a specific group.
#[utoipa::path(
    get,
    path = "/api/groups/{group_id}/task_packages/",
    tag = "task_package",
    params(
        ("group_id" = Uuid, Path, description = "The unique identifier of the group for which task packages are being fetched."),
    ),
    responses(
        (status = 200, description = "The request was successful, and a list of task packages is provided.", body = TaskPackagesResponse),
    ),
)]
#[get("/")]
pub async fn fetch_task_packages(
    path: Path<GroupPath>,
    group_repo: Data<GroupRepo>,
    _: jwt::JwtMiddleware,
    _: permission::PermissionMiddleware,
) -> HttpResponse {
    // TODO: paging
    let task_packages = match group_repo.fetch_task_packages(&path.group_id) {
        Ok(task_packages) => task_packages,
        Err(_) => return HttpResponse::NotFound().json(
            serde_json::json!({"message": "Something went wrong"})
        )
    };

    HttpResponse::Ok().json(
        TaskPackagesResponse {
            task_packages,
        }
    )
}

/// # Create Task Package Endpoint
///
/// This endpoint allows the creation of a new task package within a specific group.
#[utoipa::path(
    post,
    path = "/api/groups/{group_id}/task_packages/",
    tag = "task_package",
    request_body = CreateTaskPackageSchema,
    params(
        ("group_id" = Uuid, Path, description = "The unique identifier of the group in which the task package will be created."),
    ),
    responses(
        (status = 201, description = "The task package was successfully created. Returns information about the created task package.", body = CreatedTaskPackageResponse),
    ),
)]
#[post("/")]
pub async fn create_task_package(
    path: Path<GroupPath>,
    body: Json<CreateTaskPackageSchema>,
    app: Data<AppState>,
    data: Data<GroupRepo>,
    _: jwt::JwtMiddleware,
    _: permission::PermissionMiddleware,
) -> HttpResponse {
    let pagination = &PagingSchema{
        limit: 10,
        page: 0,
        order: OrderDir::DESC,
    };
    let mut task_ids: Vec<Uuid> = vec![];

    if body.task_doc_ids.is_some() {
        task_ids = body.task_doc_ids.as_ref().unwrap().to_owned();
    }

    let tasks = match app.mongodb.fetch_all_tasks(pagination, Some(task_ids.clone()), true).await {
        Ok(tasks) => tasks,
        Err(_) => return HttpResponse::InternalServerError().json(
            serde_json::json!({"message": "Something went wrong"})
        )
    };

    let created_task_package: CreatedTaskPackage = match data.create_task_package(&body.name, 
        &path.group_id, 
        &body.task_package_type, 
        &tasks.into_iter().map(|task| NewTempTask {
            task_doc_id: task.id,
            task_type: task.task_type,
        }).collect::<Vec<NewTempTask>>()) {
        Ok(package) => package,
        Err(err) => {
            eprint!("{err}");
            io::stdout().flush().unwrap();

            return HttpResponse::InternalServerError().json(
                serde_json::json!({"message": "Something went wrong"})
            )
        }
    };

    HttpResponse::Created().json(
        CreatedTaskPackageResponse {
            id: created_task_package.id,
            name: created_task_package.name,
            task_package_type: created_task_package.task_package_type,
            group_id: created_task_package.group_id,
            created_at: created_task_package.created_at.and_utc(),
        }
    )
}

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(
        web::scope("/task_packages")
            .service(fetch_task_packages)
            .service(create_task_package)
            .configure(task_package_id::config)
    );
}