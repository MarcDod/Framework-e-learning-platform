use chrono::{NaiveDateTime, DateTime, Utc};
use diesel::{Selectable, deserialize::Queryable, prelude::Insertable};
use mongodb::bson::Document;
use utoipa::ToSchema;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use super::util::{Visibility, TaskPackageType, AnswerState};

#[derive(ToSchema, Deserialize, Debug)]
pub struct CreateSolutionAttemptSchema {
    pub visibility: Option<Visibility>,
}

#[derive(ToSchema, Serialize, Deserialize, Debug)]
pub struct VisibilityQuery {
    pub visibility: Option<Visibility>,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::solution_attempts)]
pub struct NewSolutionAttempt {
    pub id: Uuid,
    pub user_id: Uuid,
    pub task_package_id: Uuid,
    pub visibility: Option<Visibility>,
}

#[derive(ToSchema, Debug, Selectable, Deserialize, Serialize, Queryable, Clone)]
#[diesel(table_name = crate::schema::solution_attempts)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CreatedSolutionAttempt {
    pub id: Uuid,
    pub user_id: Uuid,
    pub task_package_id: Uuid,
    pub visibility: Visibility,
    pub created_at: NaiveDateTime,
}

#[derive(Serialize, Debug, Deserialize)]
pub struct SolutionAttempt {
    pub solution_attempt: CreatedSolutionAttempt,
    pub solution_list: Vec<AnswerEntry>,
    pub state: AnswerState,
}

#[derive(ToSchema, Serialize, Deserialize, Debug)]
pub struct CreatedSolutionAttemptResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub task_package_id: Uuid,
    pub visibility: Visibility,
    pub answer_list: Vec<AnswerEntry>,
    #[schema(value_type = String)]
    pub created_at: DateTime<Utc>,
    pub state: AnswerState,
}

#[derive(ToSchema, Serialize, Deserialize, Debug)]
pub struct SolutionAttemptWithAnswerListResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub task_package_id: Uuid,
    pub visibility: Visibility,
    pub answer_list: Vec<AnswerEntry>,
    #[schema(value_type = String)]
    pub created_at: DateTime<Utc>,
    pub state: AnswerState,
}

#[derive(ToSchema, Serialize, Debug, Deserialize, PartialEq)]
pub struct AnswerEntry {
    pub answer_id: Uuid,
    pub answer_doc_id: Uuid,
    pub task_id: Uuid,
    pub task_doc_id: Uuid,
    pub task_type: String,
}

#[derive(ToSchema, Serialize, Debug)]
pub struct SolutionAttemptResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub task_package_id: Uuid,
    pub visibility: Visibility,
    #[schema(value_type = String)]
    pub created_at: DateTime<Utc>,
    pub state: AnswerState,
}

#[derive(ToSchema, Serialize, Debug)]
pub struct SolutionAttemptsResponse {
    pub solution_attempts: Vec<SolutionAttemptResponse>
}

#[derive(Deserialize)]
pub struct SolutionAttemptPath {
    pub solution_attempt_id: Uuid,
    pub task_package_id: Uuid,
    pub group_id: Uuid,
}
#[derive(Deserialize)]
pub struct SolutionAttemptGroupPath {
    pub solution_attempt_id: Uuid,
    pub group_id: Uuid,
}