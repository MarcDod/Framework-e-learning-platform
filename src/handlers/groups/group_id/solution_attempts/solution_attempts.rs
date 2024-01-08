use super::solution_attempt_id::solution_attempt_id;

use actix_web::web::{self, ServiceConfig};

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(
        web::scope("/solution_attempts")
            .configure(solution_attempt_id::config)
    );
}