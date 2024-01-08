// Documentation was created by ChatGPT
use actix_web::{web::{ServiceConfig, self, Data, Json, Query, Path}, post, get, delete, HttpResponse};
use uuid::Uuid;

use crate::{jwt, permission, repository::{group::GroupRepo, users::UsersRepo, postgres::PgRepo}, models::{groups::{GroupAddMemberSchema, NewGroupMember, GroupPath, GroupRemoveMemberSchema}, members::{MembersPagingSchema, MemberInfoResponse, MemberListResponse, MemberListWithCount, MemberListWithCountResponse}, util::{PagingSchema, OrderDir}, users::UserResponse}};

/// # Add Member to Group
///
/// ## Authentication
///
/// This route requires a valid JWT authentication token.
///
/// ## Response
///
/// The response includes a list of added members.
///
/// ## Notes
///
/// - This route adds one or more members to the specified group.
/// - The `new_members` parameter contains a list of email addresses of users to be added to the group.
/// - The response includes details about the added members, including member ID, user ID, user name, and user email.
/// - Users with the appropriate permissions can add members to a group.
/// - The added members are returned in the response.
#[utoipa::path(
    post,
    path = "/api/groups/{group_id}/members/",
    tag = "member",
    request_body = GroupAddMemberSchema,
    params(
        ("group_id" = String, Path, description = "The unique identifier of the group to which members are being added."),
    ),
    responses(
        (
            status = 201, 
            description = "Members were successfully added to the group.", 
            body = CreateGroupResponse
        ),
    ),
)]
#[post("/")]
pub async fn add_member_to_group(
    body: Json<GroupAddMemberSchema>,
    path: Path<GroupPath>,
    data: Data<GroupRepo>,
    users_repo: Data<UsersRepo>,
    _: jwt::JwtMiddleware,
    _: permission::PermissionMiddleware,
) -> HttpResponse {
    let mut added_members: Vec<MemberInfoResponse> = vec![];

    for user_to_add in &body.new_members {
        let user_id = users_repo.fetch_active_user_id_by_email(user_to_add).unwrap_or(Uuid::default());

        if user_id != Uuid::default() {
            let add_result = data.add_user_to_group(&NewGroupMember {
                user_id: &user_id,
                group_id: &path.group_id,
            });

            if add_result.is_ok() {
                let added_member = add_result.unwrap();
                added_members.push(MemberInfoResponse { 
                    id: added_member.member_id, 
                    user: UserResponse { 
                        id: added_member.user_id,
                        name: added_member.name, 
                        email: added_member.email, 
                    }
                });
            }
        }
    };
    HttpResponse::Created().json(MemberListResponse {
        members: added_members,
    })
}

/// # Remove Member from Group
///
/// ## Authentication
///
/// This route requires a valid JWT authentication token.
///
/// ## Response
///
/// The response includes a list of removed members.
///
/// ## Notes
///
/// - This route removes one or more members from the specified group.
/// - The `remove_members` parameter contains a list of user IDs to be removed from the group.
/// - The response includes details about the removed members, including member ID, user ID, user name, and user email.
/// - Users with the appropriate permissions can remove members from a group.
/// - The removed members are returned in the response.
#[utoipa::path(
    delete,
    path = "/api/groups/{group_id}/members/",
    tag = "member",
    request_body = GroupRemoveMemberSchema,
    params(
        ("group_id" = String, Path, description = "The unique identifier of the group from which members are being removed."),
    ),
    responses(
        (
            status = 200, 
            description = "Members were successfully removed from the group.", 
            body = MemberListResponse
        )
    )
)]
#[delete("/")]
pub async fn remove_member_from_group(
    body: Json<GroupRemoveMemberSchema>,
    path: Path<GroupPath>,
    data: Data<GroupRepo>,
    _: jwt::JwtMiddleware,
    _: permission::PermissionMiddleware,
) -> HttpResponse {
    let mut removed_members: Vec<MemberInfoResponse> = vec![];

    for user_to_remove in &body.remove_members {
        let remove_result = data.remove_user_from_group(&user_to_remove, &path.group_id);

        if remove_result.is_ok() {
            let removed_member = remove_result.unwrap();
            removed_members.push(MemberInfoResponse { 
                id: removed_member.member_id, 
                user: UserResponse { 
                    id: removed_member.user_id,
                    name: removed_member.name, 
                    email: removed_member.email, 
                }
            });
        }
    };
    HttpResponse::Ok().json(MemberListResponse {
        members: removed_members,
    })
}

/// # Get Group Members
///
/// ## Authentication
///
/// This route requires a valid JWT authentication token.
///
/// ## Response
///
/// The response includes a list of group members and the total count of members.
///
/// ## Notes
///
/// - This route retrieves information about the members of a specified group.
/// - The response includes details such as member ID, user ID, user name, and user email.
/// - Members can be filtered based on specific member IDs.
/// - Pagination parameters (`limit`, `page`, and `order`) allow control over the number and order of retrieved members.
/// - Users with the appropriate permissions can view the members of a group.
#[utoipa::path(
    get,
    path = "/api/groups/{group_id}/members/",
    tag = "member",
    params(
        ("group_id" = String, Path, description = "The unique identifier of the group for which members are being retrieved."),
        ("member_ids[]" = Option<String>, Query, description = "A list of member IDs to retrieve only specific members."),
        ("page" = Option<i32>, Query, description = "The page number of results if the request is paginated. Default is set to page 0."),
        ("limit" = Option<i32>, Query, description = "The maximum number of members to be returned in a single request. Default is set to 200."),
        ("order" = Option<OrderDir>, Query, description = "The order of results (ascending or descending). Default is set to descending (DESC).")
    ),
    responses(
        (
            status = 200, 
            description = "Members were successfully retrieved.", 
            body = GroupPagingResponse
        ),
    ),
)]
#[get("/")]
pub async fn get_group_members(
    query: Query<MembersPagingSchema>,
    group_repo: Data<GroupRepo>,
    path: Path<GroupPath>,
    _: jwt::JwtMiddleware,
    _: permission::PermissionMiddleware,
) -> HttpResponse {
    //TODO: compine paging and member_ids
    let pagination = &PagingSchema{
            limit: query.limit.unwrap_or(200),
            page: query.page.unwrap_or(0),
            order: query.order.unwrap_or(OrderDir::DESC),
        };

    let member_list_with_count: MemberListWithCount;

    if query.member_ids.is_none() {
        member_list_with_count = match group_repo.fetch_all_member_of_group(&path.group_id, pagination) {
            Ok(v) => v,
            Err(_) => return HttpResponse::InternalServerError().json(
                serde_json::json!({"message": "Something went wrong"})
            )
        };
    } else {
        member_list_with_count = match group_repo.fetch_all_member_of_group_in(&path.group_id, &query.member_ids.as_ref().unwrap()) {
            Ok(v) => v,
            Err(_) => return HttpResponse::InternalServerError().json(
                serde_json::json!({"message": "Something went wrong"})
            )
        }
    }

    HttpResponse::Ok().json(MemberListWithCountResponse {
        members: member_list_with_count.member_list.into_iter().map(|member| MemberInfoResponse { 
            id: member.member_id,
            user: UserResponse { 
                id: member.user_id,
                name: member.name,
                email: member.email,
            }
        }).collect(),
        total_count: member_list_with_count.total_count,
    })
}

pub fn config(cfg: &mut ServiceConfig, pgdb: PgRepo) {
    let user_repo: UsersRepo = pgdb.new_user_repo();
    cfg.service(
        web::scope("/members")
        .app_data(Data::<UsersRepo>::new(user_repo.clone()))
            .service(add_member_to_group)
            .service(remove_member_from_group)
            .service(get_group_members)
    );
}