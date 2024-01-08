use utoipa::OpenApi;

use crate::handlers;
use crate::models::auth::{LoginUserSchema, RegisterUserSchema, LoginResponse};
use crate::models::util::ErrorSchema;
use crate::models::users::UserResponse;

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::auth::auth::login,
        handlers::auth::auth::register,
        handlers::auth::auth::logout,
        handlers::auth::auth::validate,
    ), 
    components(schemas(
        LoginUserSchema,
        LoginResponse,
        RegisterUserSchema,
        ErrorSchema,
        UserResponse,
    )), 
    tags(
        (name="auth", description = "Endpoints required for user authentication."),
    ), 
)]
pub struct ApiDoc;