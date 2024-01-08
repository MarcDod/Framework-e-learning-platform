//docu written with chat-gpt
#[cfg(test)]
mod fetch_task_packages_tests {
    use actix_web::{test::{TestRequest, self}, http};
    use serde_json::Value;
    use uuid::Uuid;

    use crate::{tests::{util, test::TestRepo}, models::{permissions::{NewRessource, OptionalUserAccessType}, groups::{NewUserPermission, CreateGroupResponse, GroupInfoResponse, GroupMetaDataResponse}, util::{AccessType, PagingSchema, OrderDir}, task::TaskPagingResponse, roles::{NewRole, UpdateRolePermission, NewRolePermission, NewRoleAccessType, UpdateRoleAccesType}, task_package::TaskPackagesResponse}};

    static SCOPE: &'static str = "/api/groups/{group_id}/task_packages"; 
    
    fn get_path(group_id: &Uuid) -> String {
        format!("/api/groups/{}/task_packages/", group_id)
    }

    /// # Test: `test_fetch_task_packages`
    ///
    /// Validates the behavior of the `fetch_task_packages` handler in Actix-Web when retrieving task packages. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a test user (`created_user`) and an example group (`created_groups`) associated with `created_user`.
    ///    - Creates a task package (`task_package`) associated with the created group.
    ///    - Grants the user read access to the "task_package" resource.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `fetch_task_packages` using the test application instance with a valid access token for `created_user`.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "OK" (200), indicating a successful request.
    ///    - Verifies that the response contains the expected task package details, such as ID, name, and type.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_fetch_task_packages() {
        use crate::handlers::groups::group_id::task_packages::task_packages::fetch_task_packages;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_groups = util::create_example_groups(&test_app.group_repo, 1, created_user.id);

        let task_package = util::create_task_package(&test_app.group_repo, &vec![], &created_groups[0].id);

        util::create_permissions(&test_app.permission_repo, &test_app.group_repo, "task_package", &created_user.id, &AccessType::Read);

        let resp = test_app
        .call(
            &get_path(&created_groups[0].id),
            SCOPE,
            fetch_task_packages,
            test_app.valid_authorizate(TestRequest::get(), &created_user.id),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let response: TaskPackagesResponse = test::read_body_json(resp).await;

        assert_eq!(response.task_packages.len(), 1);
        assert_eq!(response.task_packages[0].id, task_package.id);
        assert_eq!(response.task_packages[0].name, task_package.name);
        assert_eq!(response.task_packages[0].task_package_type, task_package.task_package_type);
    }

    // # Test: `test_fetch_task_packages_invalid_auth`
    ///
    /// Validates the behavior of the `fetch_task_packages` handler in Actix-Web when attempting to retrieve task packages with invalid authentication. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a test user (`created_user`) and an example group (`created_groups`) associated with `created_user`.
    ///    - Creates a task package (`task_package`) associated with the created group.
    ///    - Grants the user read access to the "task_package" resource.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `fetch_task_packages` using the test application instance with an invalid access token for `created_user`.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "UNAUTHORIZED" (401), indicating an unauthorized request.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_fetch_task_packages_invalid_auth() {
        use crate::handlers::groups::group_id::task_packages::task_packages::fetch_task_packages;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_groups = util::create_example_groups(&test_app.group_repo, 1, created_user.id);

        let task_package = util::create_task_package(&test_app.group_repo, &vec![], &created_groups[0].id);

        util::create_permissions(&test_app.permission_repo, &test_app.group_repo, "task_package", &created_user.id, &AccessType::Read);

        let resp = test_app
        .call(
            &get_path(&created_user.id),
            SCOPE,
            fetch_task_packages,
            test_app.invalid_authorizate(TestRequest::get(), &created_user.id),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);
    }

    /// # Test: `test_fetch_task_packages_no_permission`
    ///
    /// Validates the behavior of the `fetch_task_packages` handler in Actix-Web when attempting to retrieve task packages without the required permissions. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a test user (`created_user`) and an example group (`created_groups`) associated with `created_user`.
    ///    - Creates a task package (`task_package`) associated with the created group.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `fetch_task_packages` using the test application instance with a valid access token for `created_user` but without the necessary permissions.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "FORBIDDEN" (403), indicating insufficient permissions for the request.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_fetch_task_packages_no_permission() {
        use crate::handlers::groups::group_id::task_packages::task_packages::fetch_task_packages;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_groups = util::create_example_groups(&test_app.group_repo, 1, created_user.id);

        let task_package = util::create_task_package(&test_app.group_repo, &vec![], &created_groups[0].id);

        let resp = test_app
        .call(
            &get_path(&created_user.id),
            SCOPE,
            fetch_task_packages,
            test_app.valid_authorizate(TestRequest::get(), &created_user.id),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::FORBIDDEN);
    }
}