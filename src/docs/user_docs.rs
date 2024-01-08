use utoipa::OpenApi;

use crate::handlers;
use crate::models::users::UserResponse;

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::user::user::me,
        handlers::user::user::get_my_global_permissions,
        handlers::users::user_id::user_id::fetch_user_info,
    ), 
    components(schemas(
        UserResponse,
    )), 
    tags(
        (name="user", description = "user endpoints."),
    ), 
)]
pub struct ApiDoc;