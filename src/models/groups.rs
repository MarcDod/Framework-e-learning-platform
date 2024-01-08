use chrono::{NaiveDateTime, DateTime, Utc};
use diesel::{Insertable, Selectable, Queryable, AsChangeset};
use utoipa::ToSchema;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use super::permissions::OptionalUserAccessType;
use super::util::{PagingSchema, OrderDir};

use super::util::deserialize_option_vec_uuid;

#[derive(ToSchema, Deserialize, Debug)]
pub struct CreateGroupSchema {
    pub name: String,
    pub parent: Option<Uuid>,
}

#[derive(ToSchema, Serialize, Deserialize, Debug)]
pub struct CreateGroupResponse {
    pub id: Uuid,
    pub name: String,
    #[schema(value_type = String)]
    pub created_at: DateTime<Utc>,
    pub parent: Option<Uuid>,
}

#[derive(ToSchema, Serialize, Deserialize, Debug)]
pub struct GroupMetaDataResponse {
    pub id: Uuid,
    pub created_from: Uuid,
    pub updated_from: Uuid,
    #[schema(value_type = String)]
    pub created_at: DateTime<Utc>,
    #[schema(value_type = String)]
    pub updated_at: DateTime<Utc>,
}

#[derive(ToSchema, Serialize, Debug, Deserialize, PartialEq, Eq, Hash)]
pub struct GroupInfoResponse {
    pub id: Uuid,
    pub name: String,
    pub parent: Option<Uuid>,
}

#[derive(ToSchema, Deserialize, Debug)]
pub struct GroupAddMemberSchema {
    pub new_members: Vec<String>
}

#[derive(ToSchema, Deserialize, Debug)]
pub struct GroupRemoveMemberSchema {
    pub remove_members: Vec<Uuid>
}

#[derive(ToSchema, Deserialize, Debug, Clone)]
pub struct AddPermissionSchema {
    pub new_permissions: Vec<PermissionSchema>
}

#[derive(ToSchema, Deserialize, Debug, Clone)]
pub struct GroupsSchema {
    pub paging: Option<PagingSchema>,
    #[serde(alias = "group_ids[]", default, deserialize_with = "deserialize_option_vec_uuid")]
    pub group_ids: Option<Vec<Uuid>>,
}

#[derive(ToSchema, Deserialize, Debug, Clone)]
pub struct PermissionSchema {
    pub value: String,
    pub permission_addons: Vec<OptionalUserAccessType>,
}

#[derive(ToSchema, Serialize, Deserialize, Debug)]
pub struct AddPermissionResponse {
    pub updated_permissions: Vec<String>
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::groups)]
pub struct NewGroup {
    pub id: Uuid,
    pub name: String,
    pub created_from: Uuid,
    pub updated_from: Uuid,
    pub parent: Option<Uuid>,
}

#[derive(Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::user_permissions)]
pub struct NewUserPermission {
    pub user_id: Uuid,
    pub ressource: String,
    pub group_id: Option<Uuid>,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::group_members)]
pub struct NewGroupMember<'a> {
    pub user_id: &'a Uuid,
    pub group_id: &'a Uuid,
}

#[derive(ToSchema, Debug, Selectable, Queryable, Clone)]
#[diesel(table_name = crate::schema::groups)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CreateGroup {
    pub id: Uuid,
    pub name: String,
    pub created_at: NaiveDateTime,
    pub parent: Option<Uuid>,
}

#[derive(ToSchema, Debug, Selectable, Queryable, Clone)]
#[diesel(table_name = crate::schema::role_permissions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct RolePermission {
    pub ressource: String,
}

#[derive(ToSchema, Debug, Selectable, Queryable, Clone)]
#[diesel(table_name = crate::schema::user_permissions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct GroupUserPermission {
    pub ressource: String,
    pub group_id: Option<Uuid>,
}

#[derive(ToSchema, Debug, Selectable, Queryable, Clone)]
#[diesel(table_name = crate::schema::groups)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct GroupMetaData {
    pub id: Uuid,
    pub created_from: Uuid,
    pub updated_from: Uuid,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(ToSchema, Debug, Selectable, Queryable, Clone)]
#[diesel(table_name = crate::schema::groups)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct GroupInfo {
    pub id: Uuid,
    pub name: String,
    pub parent: Option<Uuid>,
}

pub struct GroupInfoListWithCount {
    pub group_info_list: Vec<GroupInfo>,
    pub total_count: i64,
}

#[derive(ToSchema, Deserialize, Debug, Clone)]
pub struct GroupPagingSchema {
    #[serde(alias = "group_ids[]", default, deserialize_with = "deserialize_option_vec_uuid")]
    pub group_ids: Option<Vec<Uuid>>,
    pub page: Option<i32>,
    pub limit: Option<i32>,
    pub order: Option<OrderDir>,
}

#[derive(ToSchema, Serialize, Debug, Deserialize)]
pub struct GroupPagingResponse {
    pub groups: Vec<GroupInfoResponse>,
    pub total_count: i64,
}

#[derive(Deserialize)]
pub struct GroupPath {
    pub group_id: Uuid,
}

#[derive(Deserialize)]
pub struct GroupTaskPath {
    pub group_id: Uuid,
    pub task_id: Uuid,
}

#[derive(Deserialize)]
pub struct GroupQuery {
    pub group_id: Option<Uuid>,
}

#[derive(Deserialize)]
pub struct GroupTaskSolutionPath {
    pub group_id: Uuid,
    pub task_id: Uuid,
    pub user_solution_id: Uuid,
}


#[derive(Deserialize)]
pub struct GroupUserPath {
    pub group_id: Uuid,
    pub user_id: Uuid,
}