use actix_web::web::{ServiceConfig, self};

use super::{tasks::tasks, solution_attempts::solution_attempts, users::users};


pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(
        web::scope("/{task_package_id}")
            .configure(tasks::config)
            .configure(solution_attempts::config)
            .configure(users::config)
    );
}