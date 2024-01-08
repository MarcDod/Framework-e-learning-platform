use chrono::{NaiveDateTime, DateTime, Utc};
use diesel::{prelude::Insertable, Selectable, deserialize::Queryable};
use utoipa::ToSchema;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use super::util::TaskPackageType;

#[derive(ToSchema, Deserialize, Debug)]
pub struct CreateTaskPackageSchema {
    pub task_doc_ids: Option<Vec<Uuid>>,
    pub name: String,
    pub task_package_type: Option<TaskPackageType>,
}

#[derive(ToSchema, Deserialize, Debug)]
pub struct AddTaskSchema {
    pub task_doc_ids: Option<Vec<Uuid>>,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::task_packages)]
pub struct NewTaskPackage {
    pub id: Uuid,
    pub group_id: Uuid,
    pub name: String,
    pub task_package_type: Option<TaskPackageType>,
}

#[derive(Deserialize)]
pub struct TaskPackagePath {
    pub task_package_id: Uuid,
    pub group_id: Uuid,
}

#[derive(Deserialize)]
pub struct TaskPackageUserPath {
    pub task_package_id: Uuid,
    pub user_id: Uuid,
    pub group_id: Uuid,
}

#[derive(ToSchema, Debug, Selectable, Deserialize, Serialize, Queryable, Clone)]
#[diesel(table_name = crate::schema::task_packages)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CreatedTaskPackage {
    pub id: Uuid,
    pub name: String,
    pub group_id: Uuid,
    pub task_package_type: TaskPackageType,
    pub created_at: NaiveDateTime,
}

#[derive(ToSchema, Debug, Selectable, Deserialize, Serialize, Queryable, Clone)]
#[diesel(table_name = crate::schema::task_packages)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TaskPackage {
    pub id: Uuid,
    pub name: String,
    pub task_package_type: TaskPackageType,
}

#[derive(ToSchema, Serialize, Debug, Deserialize)]
pub struct CreatedTaskPackageResponse {
    pub id: Uuid,
    pub name: String,
    pub group_id: Uuid,
    pub task_package_type: TaskPackageType,
    #[schema(value_type = String)]
    pub created_at: DateTime<Utc>,
}


#[derive(ToSchema, Serialize, Debug, Deserialize)]
pub struct TaskPackagesResponse {
    pub task_packages: Vec<TaskPackage>
}

#[derive(ToSchema, Serialize, Debug, Deserialize)]
pub struct TaskPackageUserStatisticResponse {
    pub amount_tasks: usize,
    pub values: Vec<TaskPackageUserStatisticValue>,
}

#[derive(ToSchema, Serialize, Debug, Deserialize)]
pub struct TaskPackageUserStatisticValue {
    #[schema(value_type = String)]
    pub solution_attempt_id: Uuid,
    pub date: DateTime<Utc>,
    pub amount_correct: usize, 
}