//docu written with chat-gpt
#[cfg(test)]
mod create_task_packages_tests {
    use actix_web::{test::{TestRequest, self}, http};
    use serde_json::Value;
    use uuid::Uuid;

    use crate::{tests::{util, test::TestRepo}, models::{permissions::{NewRessource, OptionalUserAccessType}, groups::{NewUserPermission, CreateGroupResponse, GroupInfoResponse, GroupMetaDataResponse}, util::{AccessType, PagingSchema, OrderDir}, task::TaskPagingResponse, roles::{NewRole, UpdateRolePermission, NewRolePermission, NewRoleAccessType, UpdateRoleAccesType}, task_package::{TaskPackagesResponse, CreatedTaskPackageResponse}}};

    static SCOPE: &'static str = "/api/groups/{group_id}/task_packages"; 
    
    fn get_path(group_id: &Uuid) -> String {
        format!("/api/groups/{}/task_packages/", group_id)
    }

    fn create_body() -> Value {
        serde_json::json!({
            "name": "test",
        })
    }

    fn create_body_with_task(task_id: &Uuid) -> Value {
        serde_json::json!({
            "task_doc_ids": [task_id],
            "name": "test",
        })
    }

    /// # Test: `test_fetch_task_packages`
    ///
    /// Validates the behavior of the `create_task_package` handler in Actix-Web when creating a new task package. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a test user (`created_user`) and an example group (`created_groups`) associated with `created_user`.
    ///    - Assigns the necessary permissions (`AccessType::Create`) to the user to create task packages.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `create_task_package` using the test application instance with a valid access token for `created_user`.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "CREATED" (201), indicating that the task package was successfully created.
    ///    - Reads the response body and asserts specific attributes (e.g., `name`, `group_id`).
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_fetch_task_packages() {
        use crate::handlers::groups::group_id::task_packages::task_packages::create_task_package;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_groups = util::create_example_groups(&test_app.group_repo, 1, created_user.id);

        util::create_permissions(&test_app.permission_repo, &test_app.group_repo, "task_package", &created_user.id, &AccessType::Create);

        let resp = test_app
        .call(
            &get_path(&created_groups[0].id),
            SCOPE,
            create_task_package,
            test_app.valid_authorizate(TestRequest::post().set_json(create_body()), &created_user.id),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::CREATED);

        let response: CreatedTaskPackageResponse = test::read_body_json(resp).await;

        assert_eq!(response.name, "test");
        assert_eq!(response.group_id, created_groups[0].id);
        // TODO: assert other
    }

    /// # Test: `test_fetch_task_packages_with_task`
    ///
    /// Validates the behavior of the `create_task_package` handler in Actix-Web when creating a new task package with an associated task. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a test user (`created_user`) and an example group (`created_groups`) associated with `created_user`.
    ///    - Assigns the necessary permissions (`AccessType::Create`) to the user to create task packages.
    ///    - Creates a test task (`task`) using the MongoDB storage.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `create_task_package` using the test application instance with a valid access token for `created_user` and includes the task ID in the request body.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "CREATED" (201), indicating that the task package was successfully created.
    ///    - Reads the response body and asserts specific attributes (e.g., `name`, `group_id`).
    ///    - Retrieves tasks associated with the created task package and verifies that there is one task with the expected `task.id`.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_fetch_task_packages_with_task() {
        use crate::handlers::groups::group_id::task_packages::task_packages::create_task_package;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_groups = util::create_example_groups(&test_app.group_repo, 1, created_user.id);

        util::create_permissions(&test_app.permission_repo, &test_app.group_repo, "task_package", &created_user.id, &AccessType::Create);

        let task = util::create_task_mc(&test_app.app_state.mongodb).await;

        let resp = test_app
        .call(
            &get_path(&created_groups[0].id),
            SCOPE,
            create_task_package,
            test_app.valid_authorizate(TestRequest::post().set_json(create_body_with_task(&task.id)), &created_user.id),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::CREATED);

        let response: CreatedTaskPackageResponse = test::read_body_json(resp).await;

        assert_eq!(response.name, "test");
        assert_eq!(response.group_id, created_groups[0].id);
        // TODO: assert other

        let tasks = test_app.group_repo.fetch_tasks_from_package(&response.id, &created_groups[0].id, &None).unwrap();

        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].task_doc_id, task.id);
    }

    /// # Test: `test_fetch_task_packages_no_permission`
    ///
    /// Validates the behavior of the `create_task_package` handler in Actix-Web when a user attempts to create a task package without the necessary permissions. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a test user (`created_user`) and an example group (`created_groups`) associated with `created_user`.
    ///    - Does not assign any specific permissions to the user for creating task packages.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `create_task_package` using the test application instance with a valid access token for `created_user`.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "FORBIDDEN" (403), indicating that the user does not have the required permissions to create a task package.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_fetch_task_packages_no_permission() {
        use crate::handlers::groups::group_id::task_packages::task_packages::create_task_package;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_groups = util::create_example_groups(&test_app.group_repo, 1, created_user.id);

        let resp = test_app
        .call(
            &get_path(&created_groups[0].id),
            SCOPE,
            create_task_package,
            test_app.valid_authorizate(TestRequest::post().set_json(create_body()), &created_user.id),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::FORBIDDEN);
    }

    /// # Test: `test_fetch_task_packages_no_auth`
    ///
    /// Validates the behavior of the `create_task_package` handler in Actix-Web when a user attempts to create a task package without a valid authentication token. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a test user (`created_user`) and an example group (`created_groups`) associated with `created_user`.
    ///    - Assigns the necessary permissions to the user for creating task packages.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `create_task_package` using the test application instance with an invalid or missing access token for `created_user`.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "UNAUTHORIZED" (401), indicating that the user needs a valid authentication token to create a task package.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_fetch_task_packages_no_auth() {
        use crate::handlers::groups::group_id::task_packages::task_packages::create_task_package;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_groups = util::create_example_groups(&test_app.group_repo, 1, created_user.id);

        util::create_permissions(&test_app.permission_repo, &test_app.group_repo, "task_package", &created_user.id, &AccessType::Create);

        let resp = test_app
        .call(
            &get_path(&created_groups[0].id),
            SCOPE,
            create_task_package,
            test_app.invalid_authorizate(TestRequest::post().set_json(create_body()), &created_user.id),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);
    }
}