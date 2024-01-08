// from https://codevoweb.com/rust-jwt-authentication-with-actix-web/
use std::env;
extern crate dotenv;
use dotenv::dotenv;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub mongodb_database_url: String,
    pub mongodb_database_name: String,
    pub jwt_secret: String,
    pub use_seeder: bool,
}

impl Config {
    pub fn init() -> Config {
        dotenv().ok();
        let database_url = match env::var("DATABASE_URL") {
            Ok(v) => v.to_string(),
            Err(_) => format!("Error loading DATABASE_URL variable"),
        };
        let mongodb_database_url = match env::var("MONGODB_DATABASE_URL") {
            Ok(v) => v.to_string(),
            Err(_) => format!("Error loading MONGODB_DATABASE_URL variable"),
        };
        let mongodb_database_name = match env::var("MONGODB_DATABASE_NAME") {
            Ok(v) => v.to_string(),
            Err(_) => format!("Error loading MONGODB_DATABASE_NAME variable"),
        };
        let jwt_secret = match env::var("JWT_SECRET") {
            Ok(v) => v.to_string(),
            Err(_) => format!("Error loading JWT_SECRET variable"),
        };
        let use_seeder = match env::var("USE_SEEDER") {
            Ok(v) => v.to_string().parse::<bool>().unwrap(),
            Err(_) => false,
        };

        Config { 
            database_url,
            mongodb_database_url,
            mongodb_database_name,
            jwt_secret,
            use_seeder,
        }
    }
}