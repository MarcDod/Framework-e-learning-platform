use utoipa::ToSchema;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[derive(ToSchema, Deserialize, Debug)]
pub struct RegisterUserSchema {
    pub name: String,
    pub email: String,
    pub password: String,
}

#[derive(ToSchema, Deserialize, Debug)]
pub struct LoginUserSchema {
    pub email: String,
    pub password: String,
}

#[derive(ToSchema, Serialize, Debug, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub user_id: Uuid,
}