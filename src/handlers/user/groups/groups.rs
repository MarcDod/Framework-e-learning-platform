// Documentation was created by ChatGPT
use actix_web::{web::{self, ServiceConfig, Data, Query}, HttpResponse, HttpRequest, get, HttpMessage};
use uuid::Uuid;

use crate::{repository::{postgres::PgRepo, 
    group::{GroupRepo, READ_GROUP_INFO}, 
    permissions::PermissionsRepo}, jwt, 
    models::{groups::{GroupPagingSchema, 
        GroupPagingResponse, GroupInfoResponse, 
        GroupInfoListWithCount}, util::{PagingSchema, OrderDir, AccessType}}};

/// # Get Groups Endpoint
///
/// This endpoint allows fetching a paginated list of groups based on specified criteria.
#[utoipa::path(
    get,
    path = "/api/user/groups/",
    tag = "group",
    params(
        ("group_ids[]" = Option<String>, Query, description = "A list of group IDs to filter groups."),
        ("page" = Option<i32>, Query, description = "The page number for paginated results (default: 0)."),
        ("limit" = Option<i32>, Query, description = "The number of groups to retrieve per page (default: 200)."),
        ("order" = Option<OrderDir>, Query, description = "The order of the results (default: DESC).")
    ),
    responses(
        (
            status = 200, 
            description = "Successfully retrieved the paginated list of groups.", 
            body = GroupPagingResponse
        ),
    )
)]
#[get("/")]
pub async fn get_groups(
    req: HttpRequest,
    query: Query<GroupPagingSchema>,
    group_repo: Data<GroupRepo>,
    permission_repo: Data<PermissionsRepo>,
    _: jwt::JwtMiddleware,
) -> HttpResponse {
    let ext = req.extensions();
    let user_id = ext.get::<Uuid>().unwrap();

    let pagination = &PagingSchema{
        limit: query.limit.unwrap_or(200),
        page: query.page.unwrap_or(0),
        order: query.order.unwrap_or(OrderDir::DESC),
    };

    let groups: GroupInfoListWithCount;

    if permission_repo.user_has_permission(user_id, &READ_GROUP_INFO.to_string(), &None).unwrap_or(vec![]).contains(&AccessType::Read) {
        groups = match group_repo.fetch_all_active_groups(pagination, &query.group_ids) {
            Ok(v) => v,
            Err(_) => return HttpResponse::InternalServerError().json(
                serde_json::json!({"message": "Something went wrong"})
            )
        };
    } else {
        groups = match group_repo.fetch_groups_where_user_has_rights(user_id, pagination, &query.group_ids) {
            Ok(v) => v,
            Err(_) => return HttpResponse::InternalServerError().json(
                serde_json::json!({"message": "Something went wrong"})
            )
        };
    }

    HttpResponse::Ok().json(GroupPagingResponse {
        groups: groups.group_info_list.into_iter().map(|group| GroupInfoResponse { 
            id: group.id,
            name: group.name,
            parent: group.parent,
        }).collect(),
        total_count: groups.total_count,
    })
}


pub fn config(cfg: &mut ServiceConfig, pgdb: PgRepo) {
    let group_repo = pgdb.new_group_repo();
    cfg.service(
        web::scope("/groups")
            .app_data(Data::<GroupRepo>::new(group_repo.clone()))
            .service(get_groups)
    );
}