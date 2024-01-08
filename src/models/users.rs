use chrono::NaiveDateTime;
use diesel::{Queryable, Selectable, Insertable};
use serde::{Serialize, Deserialize};
use utoipa::ToSchema;
use uuid::Uuid;

use super::util::State;


#[derive(Queryable, Selectable, Clone, Debug)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub password: String,
    pub state: State,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UserPath {
    pub user_id: Uuid,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UserQuery {
    pub user_id: Option<Uuid>,
}

#[derive(Debug, Selectable, Queryable)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UserPassword {
    pub id: Uuid,
    pub password: String,
}

#[derive(ToSchema, Serialize, Debug, Clone, Deserialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub name: String,
    pub email: String,
}

#[derive(Debug, Selectable, Queryable, Clone)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UserInfo {
    pub id: Uuid,
    pub name: String,
    pub email: String,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::users)]
pub struct NewUser<'a> {
    pub password: &'a str,
    pub email: &'a str,
    pub name: &'a str,
}

