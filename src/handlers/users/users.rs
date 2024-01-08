use actix_web::web::{ServiceConfig, self, Data};

use crate::repository::{postgres::PgRepo, users::UsersRepo};

use super::user_id::user_id;

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(
        web::scope("/users")
            .configure(user_id::config)
    );
}