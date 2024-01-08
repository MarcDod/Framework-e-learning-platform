use utoipa::OpenApi;

use crate::models::groups::{AddPermissionSchema, CreateGroupSchema, 
    CreateGroupResponse, GroupInfoResponse, GroupMetaDataResponse, 
    GroupAddMemberSchema, GroupRemoveMemberSchema, PermissionSchema, AddPermissionResponse, GroupsSchema,
    GroupPagingResponse};
use crate::models::permissions::{OptionalUserAccessType, PermissionListResponse, PermissionInfo, PermissionListResponseWithCount};
use crate::handlers;
use crate::models::util::{PagingSchema, OrderDir, State};

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::groups::groups::create_group,
        handlers::user::groups::groups::get_groups,
        handlers::groups::group_id::group_id::get_group,
        handlers::groups::group_id::group_id::get_group_meta_data,
        handlers::groups::group_id::group_id::delete_group,
        handlers::user::user::get_my_global_permissions,
        handlers::users::user_id::user_id::add_permissions_to_user,
        handlers::groups::group_id::users::user_id::user_id::get_group_permissions_from_user,
    ), 
    components(schemas(
        CreateGroupSchema,
        CreateGroupResponse,
        GroupInfoResponse,
        GroupMetaDataResponse,
        GroupAddMemberSchema,
        AddPermissionSchema,
        AddPermissionResponse,
        PermissionSchema,
        GroupsSchema,
        PagingSchema,
        OrderDir,
        GroupPagingResponse,
        GroupInfoResponse,
        PermissionListResponse,
        PermissionInfo,
        PermissionListResponseWithCount,
        GroupRemoveMemberSchema,
        State,
        OptionalUserAccessType
    )), 
    tags(
        (name="group", description = "Groups in our framework serve as folders, facilitating the organization of users and task packages."),
    ), 
)]
pub struct ApiDoc;