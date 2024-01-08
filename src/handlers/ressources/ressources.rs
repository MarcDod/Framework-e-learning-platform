// Documentation was created by ChatGPT
use actix_web::{web::{ServiceConfig, self, Data, Query}, HttpResponse, get};

use crate::{models::{util::{PagingSchema, OrderDir}, permissions::RessourcesPagingSchema}, jwt, permission, repository::permissions::PermissionsRepo};


/// # Fetch Resources Endpoint
///
/// This endpoint retrieves a list of resources along with their access types.
#[utoipa::path(
    get,
    path = "/api/ressources/",
    tag = "permission",
    params(
        ("ressources[]" = Option<String>, Query, description = "An array of resource names to filter the results."),
        ("page" = Option<i32>, Query, description = "The page number for pagination (default: 0)."),
        ("limit" = Option<i32>, Query, description = "The maximum number of resources to be returned (default: 200)."),
        ("order" = Option<OrderDir>, Query, description = "The order in which resources should be returned (default: DESC)."),
    ),
    responses(
        (status = 200, description = "The request was successful, and a list of resources with their access types is provided.", body = RessourceAndAccessTypesListWithCount),
    ),
)]
#[get("/")]
pub async fn fetch_ressources(
    query: Query<RessourcesPagingSchema>,
    permission_repo: Data<PermissionsRepo>,
    _: jwt::JwtMiddleware,
    _: permission::PermissionMiddleware,
) -> HttpResponse {
    let pagination = &PagingSchema{
        limit: query.limit.unwrap_or(200),
        page: query.page.unwrap_or(0),
        order: query.order.unwrap_or(OrderDir::DESC),
    };

    let ressources = match permission_repo.fetch_ressources_with_access_type(pagination, &query.ressources) {
        Ok(ressources) => ressources,
        Err(_) => return HttpResponse::InternalServerError().json(
            serde_json::json!({"message": "Something went wrong"})
        )
    };

    HttpResponse::Ok().json(
        serde_json::json!(ressources)
    )
}

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(
        web::scope("/ressources")
            .service(fetch_ressources)
    );
}