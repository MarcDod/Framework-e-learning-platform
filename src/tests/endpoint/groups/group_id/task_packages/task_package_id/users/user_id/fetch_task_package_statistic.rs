//TODO: test statistic

#[cfg(test)]
mod fetch_task_packages_statistic_tests {
    use actix_web::{test::{TestRequest, self}, http};
    use serde_json::Value;
    use uuid::Uuid;

    use crate::{tests::{util, test::TestRepo}, models::{permissions::{NewRessource, OptionalUserAccessType}, groups::{NewUserPermission, CreateGroupResponse, GroupInfoResponse, GroupMetaDataResponse}, util::{AccessType, PagingSchema, OrderDir}, task::TaskPagingResponse, roles::{NewRole, UpdateRolePermission, NewRolePermission, NewRoleAccessType, UpdateRoleAccesType}, task_package::TaskPackagesResponse}};

    static SCOPE: &'static str = "/api/groups/{group_id}/task_packages/{task_package_id}/users/{user_id}"; 
    
    fn get_path(group_id: &Uuid, task_package_id: &Uuid, user_id: &Uuid) -> String {
        format!("/api/groups/{}/task_packages/{}/users/{}/statistic", group_id, task_package_id, user_id)
    }

}