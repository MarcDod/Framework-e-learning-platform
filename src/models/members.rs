use diesel::prelude::Queryable;
use utoipa::ToSchema;

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use super::{users::UserResponse, util::OrderDir};

use super::util::deserialize_option_vec_uuid;

#[derive(ToSchema, Debug, Clone, Serialize, Deserialize)]
pub struct MemberInfoResponse {
    pub id: Uuid,
    pub user: UserResponse,
}

#[derive(ToSchema, Deserialize, Debug, Clone)]
pub struct MembersPagingSchema {
    #[serde(alias = "member_ids[]", default, deserialize_with = "deserialize_option_vec_uuid")]
    pub member_ids: Option<Vec<Uuid>>,
    pub page: Option<i32>,
    pub limit: Option<i32>,
    pub order: Option<OrderDir>,
}

#[derive(ToSchema, Serialize, Deserialize, Debug)]
pub struct MemberListResponse {
    pub members: Vec<MemberInfoResponse>,
}

#[derive(ToSchema, Serialize, Deserialize, Debug)]
pub struct MemberListWithCountResponse {
    pub members: Vec<MemberInfoResponse>,
    pub total_count: i64,
}

#[derive(Queryable)]
pub struct MemberInfo {
    pub member_id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub email: String,
}

pub struct MemberListWithCount {
    pub member_list: Vec<MemberInfo>,
    pub total_count: i64,
}