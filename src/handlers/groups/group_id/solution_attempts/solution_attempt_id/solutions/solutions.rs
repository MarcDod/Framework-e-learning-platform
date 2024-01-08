use actix_web::web::{ServiceConfig, self};

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(
        web::scope("/solutions")
    );
}