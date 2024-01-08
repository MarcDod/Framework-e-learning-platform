use diesel::{Selectable, prelude::{Queryable, Insertable}, query_builder::AsChangeset};
use utoipa::ToSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::util::{deserialize_option_vec_string, OrderDir, AccessType};

#[derive(ToSchema, Deserialize, Debug, Clone)]
pub struct PermissionRequest {
    #[serde(alias = "ressources[]", default, deserialize_with = "deserialize_option_vec_string")]
    pub ressources: Option<Vec<String>>,
    pub page: Option<i32>,
    pub limit: Option<i32>,
    pub order: Option<OrderDir>,
    #[serde(default)]
    pub group_only: bool,
}

#[derive(ToSchema, Deserialize, Debug, Clone)]
pub struct RessourcesPagingSchema {
    #[serde(
        alias = "ressources[]",
        default,
        deserialize_with = "deserialize_option_vec_string"
    )]
    pub ressources: Option<Vec<String>>,
    pub page: Option<i32>,
    pub limit: Option<i32>,
    pub order: Option<OrderDir>,
}

#[derive(ToSchema, Deserialize, Queryable, Serialize, Debug, Clone)]
pub struct PermissionInfo {
    pub key_value: String,
    pub key_name: String,
    pub access_types: Vec<UserAccessType>,
    pub group_id: Option<Uuid>,
}

#[derive(ToSchema, Debug, Selectable, Queryable, Clone, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::user_access_types)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UserAccessType {
    pub access_type: AccessType,
    pub permission: bool,
    pub set_permission: bool,
    pub set_set_permission: bool,
}

#[derive(ToSchema, Debug, Selectable, Queryable, Clone, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::role_access_types)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct RoleAccesType {
    pub access_type: AccessType,
    pub set_permission: bool,
    pub set_set_permission: bool,
    pub permission: bool,
}

pub struct PermissionInfoListWithCount {
    pub permission_list: Vec<PermissionInfo>,
    pub total_count: i64,
}

#[derive(ToSchema, Serialize, Deserialize, Debug, Clone)]
pub struct RessourceListWithCount {
    pub ressources: Vec<Ressource>,
    pub total_count: i64,
}

#[derive(ToSchema, Serialize, Deserialize, Debug, Clone)]
pub struct RessourceAndAccessTypesListWithCount {
    pub ressources: Vec<RessourceWithAccessTypes>,
    pub total_count: i64,
}

#[derive(ToSchema, Deserialize, Debug, Clone, Serialize)]
pub struct PermissionListResponse {
    pub permission_list: Vec<PermissionInfo>
}

#[derive(ToSchema, Deserialize, Debug, Clone, Serialize)]
pub struct PermissionListResponseWithCount {
    pub permission_list: Vec<PermissionInfo>,
    pub total_count: i64,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::ressources)]
pub struct NewRessource<'a> {
    pub key_name: &'a String,
    pub key_value: &'a String,
}


#[derive(ToSchema, Debug, Selectable, Queryable, Clone, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::ressources)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Ressource {
    pub key_value: String,
    pub key_name: String
}

#[derive(ToSchema, Debug, Clone, Serialize, Deserialize)]
pub struct RessourceWithAccessTypes {
    pub key_value: String,
    pub key_name: String,
    pub access_types: Vec<AccessType>,
}

#[derive(ToSchema, Debug, Clone, Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::user_access_types)]
pub struct NewUserAccessType {
    pub access_type: AccessType,
    pub set_permission: Option<bool>,
    pub set_set_permission: Option<bool>,
    pub permission: Option<bool>,
    pub user_permission_id: Uuid,
}

#[derive(ToSchema, Debug, Clone, Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::user_access_types)]
pub struct UpdateUserAccessType {
    pub set_permission: Option<bool>,
    pub set_set_permission: Option<bool>,
    pub permission: Option<bool>,
}

#[derive(ToSchema, Deserialize, Debug, Clone)]
pub struct OptionalUserAccessType {
    pub access_type: AccessType,
    pub set_permission: Option<bool>,
    pub set_set_permission: Option<bool>,
    pub permission: Option<bool>,
}