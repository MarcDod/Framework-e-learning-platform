use chrono::{NaiveDateTime, DateTime, Utc};
use diesel::{prelude::Insertable, Selectable, deserialize::Queryable};
use mongodb::bson::{Document, DateTime as BDateTime};
use serde::{Serialize, Deserialize};
use utoipa::ToSchema;
use uuid::Uuid;

use super::util::AnswerState;

#[derive(ToSchema, Serialize, Deserialize, Debug, Clone)]
pub struct AnswerDocIdWithString {
    #[serde(rename = "_id")]
    pub id: String,
    pub solution: Document,
    pub updated_at: BDateTime,
}

#[derive(ToSchema, Serialize, Deserialize, Debug, Clone)]
pub struct AnswerDoc {
    pub id: Uuid,
    pub solution: Document,
    pub updated_at: NaiveDateTime,
}

#[derive(ToSchema, Deserialize, Debug)]
pub struct UpdateAnswerSchema {
    pub solution: Document,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::answers)]
pub struct NewAnswer<'a> {
    pub solution_attempt_id: &'a Uuid,
    pub answer_doc_id: &'a Uuid,
    pub task_id: &'a Uuid,
    pub state: &'a AnswerState,
    pub created_from: &'a Uuid,
}

#[derive(ToSchema, Debug, Selectable, Deserialize, Serialize, Queryable, Clone)]
#[diesel(table_name = crate::schema::answers)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CreatedAnswer {
    pub id: Uuid,
    pub solution_attempt_id: Uuid,
    pub answer_doc_id: Uuid,
    pub task_id: Uuid,
    pub created_from: Uuid,
}


#[derive(ToSchema, Serialize, Debug, Deserialize)]
pub struct AddAnswerResponse {
    pub id: Uuid,
    pub answer_doc_id: Uuid,
    pub solution: Document,
    pub task_id: Uuid,
    pub user_id: Uuid,
    pub created_from: Uuid,
    #[schema(value_type = String)]
    pub created_at: DateTime<Utc>,
    #[schema(value_type = String)]
    pub updated_at: DateTime<Utc>,
}

#[derive(ToSchema, Serialize, Debug, Deserialize)]
pub struct UpdatedAnswerResponse {
    pub id: Uuid,
    pub answer_doc_id: Uuid,
    pub solution: Document,
    pub task_id: Uuid,
    pub created_from: Uuid,
    #[schema(value_type = String)]
    pub updated_at: DateTime<Utc>,
}

#[derive(ToSchema, Serialize, Debug, Deserialize)]
pub struct AnswerResponse {
    pub id: Uuid,
    pub answer_doc_id: Uuid,
    pub answer_doc: Option<Document>,
    pub state: AnswerState,
    pub task_id: Uuid,
    pub correct: bool,
    pub created_from: Uuid,
    #[schema(value_type = String)]
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(ToSchema, Debug, Selectable, Deserialize, Serialize, Queryable, Clone)]
#[diesel(table_name = crate::schema::answers)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Answer {
    pub id: Uuid,
    pub correct: bool,
    pub solution_attempt_id: Uuid,
    pub answer_doc_id: Uuid,
    pub task_id: Uuid,
    pub state: AnswerState,
    pub created_from: Uuid,
}

#[derive(ToSchema, Serialize, Debug, Deserialize)]
pub struct AnswersResponse {
    pub answers: Vec<Answer>,
}

#[derive(Deserialize)]
pub struct AnswerPath {
    pub answer_id: Uuid,
    pub group_id: Uuid,
}

#[derive(Deserialize)]
pub struct AnswerUserPath {
    pub answer_id: Uuid,
    pub user_id: Uuid,
    pub group_id: Uuid,
    pub solution_attempt_id: Uuid,
}