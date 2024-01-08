use utoipa::OpenApi;

use crate::models::members::{MemberInfoResponse, 
    MembersPagingSchema, MemberListResponse, MemberListWithCountResponse};
use crate::handlers;

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::groups::group_id::members::members::add_member_to_group,
        handlers::groups::group_id::members::members::remove_member_from_group,
        handlers::groups::group_id::members::members::get_group_members,
    ), 
    components(schemas(
        MemberInfoResponse, 
        MembersPagingSchema, 
        MemberListResponse,
        MemberListWithCountResponse,
    )), 
    tags(
        (name="member", description = "Group members."),
    ), 
)]
pub struct ApiDoc;