use actix_cors::Cors;
use actix_web::{
    web::{self, Data},
    App, HttpResponse, HttpServer,
};
use config::Config;
use permission_config::PermissionConfig;
use repository::{postgres::PgRepo, mongodb::MongoDbRepo};
use serde_json::json;
use utoipa_swagger_ui::SwaggerUi;

use utoipa::OpenApi;

mod config;
mod docs;
mod handlers;
mod jwt;
mod models;
mod permission;
mod permission_config;
mod repository;
mod schema;
mod seeder;

#[cfg(test)]
mod tests;

#[derive(Clone)]
pub struct AppState {
    env: Config,
    pgdb: PgRepo,
    mongodb: MongoDbRepo,
    permission_config: PermissionConfig,
}

impl AppState {
    pub async fn init() -> AppState {
        let env = config::Config::init();
        let pgdb = repository::postgres::PgRepo::establish_connection(env.database_url.to_string());
        let mongodb: MongoDbRepo = repository::mongodb::MongoDbRepo::establish_connection(&env.mongodb_database_url, &env.mongodb_database_name).await;
        let permission_config = PermissionConfig::new();
        AppState {
            env,
            pgdb,
            mongodb,
            permission_config,
        }
    }
}

async fn not_found() -> HttpResponse {
    HttpResponse::NotFound().json(json!({
        "message": "Ressource not found",
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let app_state = AppState::init().await;

    if app_state.env.use_seeder {
        seeder::seeder::fill_seeder().await;
    }

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(app_state.clone()))
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", docs::docs::ApiDoc::openapi()),
            )
            .configure(|cfg| handlers::api::config(cfg, app_state.pgdb.clone()))
            .default_service(web::route().to(not_found))
            .wrap(actix_web::middleware::Logger::default())
            .wrap(Cors::permissive())
    })
    .bind(("localhost", 8000))?
    .run()
    .await
}
