use diesel::deserialize::Queryable;
use diesel::prelude::Insertable;
use diesel::Selectable;
use mongodb::bson::Document;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::repository::mongodb::SchemaDoc;

use super::util::deserialize_option_vec_string;
use super::util::deserialize_option_vec_uuid;
use super::util::OrderDir;
use super::util::State;

#[derive(ToSchema, Deserialize, Debug, Clone)]
pub struct CreateTaskRequest {
    pub task_type: String,
    pub task: Document,
    pub solution: Document,
}

#[derive(ToSchema, Deserialize, Debug, Clone)]
pub struct CreateSchemaRequest {
    pub task_type: String,
    pub task_schema: Document,
    pub solution_schema: Document,
}

#[derive(ToSchema, Deserialize, Serialize, Debug, Clone)]
pub struct CreateTaskResponse {
    pub task: TaskDoc,
}

#[derive(ToSchema, Serialize, Deserialize, Debug, Clone)]
pub struct TaskDocIdString {
    #[serde(rename = "_id")]
    pub id: String,
    pub task_type: String,
    pub task: Document,
    pub state: String,
    pub solution: Document,
}

#[derive(ToSchema, Serialize, Deserialize, Debug, Clone)]
pub struct TaskDoc {
    pub id: Uuid,
    pub task_type: String,
    pub task: Document,
    pub state: State,
    pub solution: Document,
}

#[derive(ToSchema, Serialize, Deserialize, Debug, Clone)]
pub struct TaskResponse {
    pub id: Uuid,
    pub task_type: String,
    pub task: Document,
    pub state: State,
}

#[derive(ToSchema, Serialize, Deserialize, Debug, Clone)]
pub struct TaskSolution {
    pub solution: Document,
}


#[derive(ToSchema, Serialize, Debug, Deserialize)]
pub struct TaskPagingResponse {
    pub tasks: Vec<TaskResponse>,
    pub total_count: u64,
}

#[derive(ToSchema, Serialize, Debug, Deserialize)]
pub struct SchemaPagingResponse {
    pub schemas: Vec<SchemaDoc>,
    pub total_count: u64,
}

#[derive(ToSchema, Serialize, Debug, Deserialize)]
pub struct TasksResponse {
    pub tasks: Vec<TaskResponse>,
}

#[derive(ToSchema, Deserialize, Debug, Clone)]
pub struct TaskPagingSchema {
    #[serde(
        alias = "task_ids[]",
        default,
        deserialize_with = "deserialize_option_vec_uuid"
    )]
    pub task_ids: Option<Vec<Uuid>>,
    pub page: Option<i32>,
    pub limit: Option<i32>,
    pub order: Option<OrderDir>,
}

#[derive(ToSchema, Deserialize, Debug, Clone)]
pub struct SchemaPagingSchema {
    #[serde(
        alias = "task_types[]",
        default,
        deserialize_with = "deserialize_option_vec_string"
    )]
    pub task_types: Option<Vec<String>>,
    pub page: Option<i32>,
    pub limit: Option<i32>,
    pub order: Option<OrderDir>,
}

#[derive(ToSchema, Deserialize)]
pub struct TaskPath {
    pub task_id: Uuid,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::tasks)]
pub struct NewTask<'a> {
    pub task_doc_id: &'a Uuid,
    pub task_package_id: &'a Uuid,
    pub task_type: &'a String,
}

#[derive(ToSchema, Deserialize, Debug, Clone)]
pub struct TaskTypeFilter {
    #[serde(
        alias = "task_types[]",
        default,
        deserialize_with = "deserialize_option_vec_string"
    )]
    pub task_types: Option<Vec<String>>,
}

pub struct NewTempTask {
    pub task_doc_id: Uuid,
    pub task_type: String,
}

#[derive(ToSchema, Debug, Selectable, Serialize, Deserialize, Queryable, Clone)]
#[diesel(table_name = crate::schema::tasks)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Task {
    pub id: Uuid,
    pub task_package_id: Uuid,
    pub task_doc_id: Uuid,
    pub task_type: String,
}

#[derive(ToSchema, Deserialize, Debug)]
pub struct AddTasksToPackageSchema {
    pub task_doc_ids: Vec<Uuid>,
}

#[derive(ToSchema, Deserialize, Debug)]
pub struct RemoveTasksFromPackageSchema {
    pub task_ids: Vec<Uuid>,
}

#[derive(ToSchema, Serialize, Deserialize, Debug)]
pub struct AddedTasksToPackageResponse {
    pub added_tasks: Vec<Task>,
}

#[derive(ToSchema, Serialize, Debug)]
pub struct RemoveTasksFromPackageResponse {
    pub removed_tasks: Vec<Task>,
}

#[derive(ToSchema, Serialize, Deserialize, Debug)]
pub struct TasksFromPackageResponse {
    pub tasks: Vec<Task>,
}

#[derive(ToSchema, Serialize, Debug)]
pub struct SchemasResponse {
    pub tasks: Vec<Task>,
}
