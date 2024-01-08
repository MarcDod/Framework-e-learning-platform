use actix_web::web::{ServiceConfig, self};

use super::answer_id::answer_id;

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(
        web::scope("/answers")
            .configure(answer_id::config)
    );
}