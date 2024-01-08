use chrono::{NaiveDateTime, DateTime, Utc};
use diesel::{Selectable, deserialize::Queryable, prelude::Insertable, query_builder::AsChangeset};
use utoipa::ToSchema;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use super::util::AccessType;

pub struct UpdateRolePermission {
    pub role_permission: NewRolePermission,
    pub role_access_types: Vec<UpdateRoleAccesType>,
}

pub struct UpdateRoleAccesType {
    pub access_type: AccessType,
    pub set_permission: Option<bool>,
    pub set_set_permission: Option<bool>,
    pub permission: Option<bool>,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::roles)]
pub struct NewRole<'a> {
    pub name: &'a String,
    pub value_key: &'a String,
}

#[derive(ToSchema, Debug, Clone, Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::role_access_types)]
pub struct NewRoleAccessType {
    pub access_type: AccessType,
    pub set_permission: Option<bool>,
    pub set_set_permission: Option<bool>,
    pub permission: Option<bool>,
    pub role_permission_id: Uuid,
}

#[derive(Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::role_permissions)]
pub struct NewRolePermission {
    pub role: String,
    pub ressource: String,
}