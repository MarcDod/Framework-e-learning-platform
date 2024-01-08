//docu written with chat-gpt
#[cfg(test)]
mod add_task_to_task_package_tests {
    use actix_web::{test::{TestRequest, self}, http};
    use serde_json::Value;
    use uuid::Uuid;

    use crate::{tests::{util, test::TestRepo}, models::{permissions::{NewRessource, OptionalUserAccessType}, groups::{NewUserPermission, CreateGroupResponse, GroupInfoResponse, GroupMetaDataResponse}, util::{AccessType, PagingSchema, OrderDir}, task::{TaskPagingResponse, AddedTasksToPackageResponse}, roles::{NewRole, UpdateRolePermission, NewRolePermission, NewRoleAccessType, UpdateRoleAccesType}, task_package::{TaskPackagesResponse, CreatedTaskPackageResponse}}};

    static SCOPE: &'static str = "/api/groups/{group_id}/task_packages/{task_package_id}/tasks"; 
    
    fn get_path(group_id: &Uuid, task_package_id: &Uuid) -> String {
        format!("/api/groups/{}/task_packages/{}/tasks/", group_id, task_package_id)
    }

    fn create_body(task_id: &Uuid) -> Value {
        serde_json::json!({
            "task_doc_ids": [task_id],
        })
    }

    fn create__one_illegal_body(task_id: &Uuid) -> Value {
        serde_json::json!({
            "task_doc_ids": [task_id, Uuid::new_v4()],
        })
    }

    /// # Test: `test_add_task_to_task_package`
    ///
    /// Validates the behavior of the `add_tasks_to_package` handler in Actix-Web when adding tasks to a task package. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a test user (`created_user`) and an example group (`created_groups`) associated with `created_user`.
    ///    - Creates a task package (`task_package`) associated with the group.
    ///    - Grants the necessary permissions (`AccessType::Write`) to the user for adding tasks to the task package.
    ///    - Creates a test task (`task`).
    ///
    /// 2. Execution:
    ///    - Makes an API call to `add_tasks_to_package` using the test application instance, providing a valid authorization token for `created_user` and the task ID.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "OK" (200), indicating a successful operation.
    ///    - Validates the response payload, checking that the task has been added to the task package.
    ///    - Retrieves the tasks associated with the task package and ensures that the added task is present.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_add_task_to_task_package() {
        use crate::handlers::groups::group_id::task_packages::task_package_id::tasks::tasks::add_tasks_to_package;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_groups = util::create_example_groups(&test_app.group_repo, 1, created_user.id);

        let task_package = util::create_task_package(&test_app.group_repo, &vec![], &created_groups[0].id);

        util::create_permissions(&test_app.permission_repo, &test_app.group_repo, "task_package_task", &created_user.id, &AccessType::Write);

        let task = util::create_task_mc(&test_app.app_state.mongodb).await;

        let resp = test_app
        .call(
            &get_path(&created_groups[0].id, &task_package.id),
            SCOPE,
            add_tasks_to_package,
            test_app.valid_authorizate(TestRequest::post().set_json(create_body(&task.id)), &created_user.id),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let response: AddedTasksToPackageResponse = test::read_body_json(resp).await;

        assert_eq!(response.added_tasks.len(), 1);
        assert_eq!(response.added_tasks[0].task_package_id, task_package.id);
        assert_eq!(response.added_tasks[0].task_doc_id, task.id);

        let tasks = test_app.group_repo.fetch_tasks_from_package(&task_package.id, &created_groups[0].id, &None).unwrap();

        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].task_doc_id, task.id);
    }

    /// # Test: `test_add_task_to_task_package_one_illegal`
    ///
    /// Validates the behavior of the `add_tasks_to_package` handler in Actix-Web when adding tasks to a task package, including one illegal task. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a test user (`created_user`) and an example group (`created_groups`) associated with `created_user`.
    ///    - Creates a task package (`task_package`) associated with the group.
    ///    - Grants the necessary permissions (`AccessType::Write`) to the user for adding tasks to the task package.
    ///    - Creates a test task (`task`).
    ///
    /// 2. Execution:
    ///    - Makes an API call to `add_tasks_to_package` using the test application instance, providing a valid authorization token for `created_user` and the task ID (including one illegal task ID).
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "OK" (200), indicating a successful operation.
    ///    - Validates the response payload, checking that the legal task has been added to the task package.
    ///    - Retrieves the tasks associated with the task package and ensures that the added task is present.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_add_task_to_task_package_one_illegal() {
        use crate::handlers::groups::group_id::task_packages::task_package_id::tasks::tasks::add_tasks_to_package;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_groups = util::create_example_groups(&test_app.group_repo, 1, created_user.id);

        let task_package = util::create_task_package(&test_app.group_repo, &vec![], &created_groups[0].id);

        util::create_permissions(&test_app.permission_repo, &test_app.group_repo, "task_package_task", &created_user.id, &AccessType::Write);

        let task = util::create_task_mc(&test_app.app_state.mongodb).await;

        let resp = test_app
        .call(
            &get_path(&created_groups[0].id, &task_package.id),
            SCOPE,
            add_tasks_to_package,
            test_app.valid_authorizate(TestRequest::post().set_json(create__one_illegal_body(&task.id)), &created_user.id),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let response: AddedTasksToPackageResponse = test::read_body_json(resp).await;

        assert_eq!(response.added_tasks.len(), 1);
        assert_eq!(response.added_tasks[0].task_package_id, task_package.id);
        assert_eq!(response.added_tasks[0].task_doc_id, task.id);

        let tasks = test_app.group_repo.fetch_tasks_from_package(&task_package.id, &created_groups[0].id, &None).unwrap();

        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].task_doc_id, task.id);
    }

    /// # Test: `test_add_task_to_task_package_wrong_group`
    ///
    /// Validates the behavior of the `add_tasks_to_package` handler in Actix-Web when attempting to add tasks to a task package in a different group. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a test user (`created_user`) and an example group (`created_groups`) associated with `created_user`.
    ///    - Creates a task package (`task_package`) associated with the group.
    ///    - Grants the necessary permissions (`AccessType::Write`) to the user for adding tasks to the task package.
    ///    - Creates a test task (`task`).
    ///
    /// 2. Execution:
    ///    - Makes an API call to `add_tasks_to_package` using the test application instance, providing a valid authorization token for `created_user` and the task ID.
    ///    - Uses a different group ID (not associated with the task package) in the API call.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "OK" (200), indicating a successful operation.
    ///    - Validates the response payload, checking that no tasks have been added to the task package.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_add_task_to_task_package_wrong_group() {
        use crate::handlers::groups::group_id::task_packages::task_package_id::tasks::tasks::add_tasks_to_package;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_groups = util::create_example_groups(&test_app.group_repo, 1, created_user.id);

        let task_package = util::create_task_package(&test_app.group_repo, &vec![], &created_groups[0].id);

        util::create_permissions(&test_app.permission_repo, &test_app.group_repo, "task_package_task", &created_user.id, &AccessType::Write);

        let task = util::create_task_mc(&test_app.app_state.mongodb).await;

        let resp = test_app
        .call(
            &get_path(&Uuid::new_v4(), &task_package.id),
            SCOPE,
            add_tasks_to_package,
            test_app.valid_authorizate(TestRequest::post().set_json(create_body(&task.id)), &created_user.id),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let response: AddedTasksToPackageResponse = test::read_body_json(resp).await;

        assert_eq!(response.added_tasks.len(), 0);
    }

    /// # Test: `test_add_task_to_task_package_task_package`
    ///
    /// Validates the behavior of the `add_tasks_to_package` handler in Actix-Web when attempting to add tasks to a task package using an invalid task package ID. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a test user (`created_user`) and an example group (`created_groups`) associated with `created_user`.
    ///    - Creates a task package (`task_package`) associated with the group.
    ///    - Grants the necessary permissions (`AccessType::Write`) to the user for adding tasks to the task package.
    ///    - Creates a test task (`task`).
    ///
    /// 2. Execution:
    ///    - Makes an API call to `add_tasks_to_package` using the test application instance, providing a valid authorization token for `created_user` and the task ID.
    ///    - Uses an invalid task package ID (not associated with the group) in the API call.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "OK" (200), indicating a successful operation.
    ///    - Validates the response payload, checking that no tasks have been added to the task package.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_add_task_to_task_package_task_packe() {
        use crate::handlers::groups::group_id::task_packages::task_package_id::tasks::tasks::add_tasks_to_package;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_groups = util::create_example_groups(&test_app.group_repo, 1, created_user.id);

        let task_package = util::create_task_package(&test_app.group_repo, &vec![], &created_groups[0].id);

        util::create_permissions(&test_app.permission_repo, &test_app.group_repo, "task_package_task", &created_user.id, &AccessType::Write);

        let task = util::create_task_mc(&test_app.app_state.mongodb).await;

        let resp = test_app
        .call(
            &get_path(&created_groups[0].id, &Uuid::new_v4()),
            SCOPE,
            add_tasks_to_package,
            test_app.valid_authorizate(TestRequest::post().set_json(create_body(&task.id)), &created_user.id),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let response: AddedTasksToPackageResponse = test::read_body_json(resp).await;

        assert_eq!(response.added_tasks.len(), 0);
    }

    /// # Test: `test_add_task_to_task_package_no_auth`
    ///
    /// Validates the behavior of the `add_tasks_to_package` handler in Actix-Web when attempting to add tasks to a task package without valid authorization. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a test user (`created_user`) and an example group (`created_groups`) associated with `created_user`.
    ///    - Creates a task package (`task_package`) associated with the group.
    ///    - Grants the necessary permissions (`AccessType::Write`) to the user for adding tasks to the task package.
    ///    - Creates a test task (`task`).
    ///
    /// 2. Execution:
    ///    - Makes an API call to `add_tasks_to_package` using the test application instance, providing an invalid authorization token for `created_user` and the task ID.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "UNAUTHORIZED" (401), indicating a lack of valid authorization.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_add_task_to_task_package_no_auth() {
        use crate::handlers::groups::group_id::task_packages::task_package_id::tasks::tasks::add_tasks_to_package;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_groups = util::create_example_groups(&test_app.group_repo, 1, created_user.id);

        let task_package = util::create_task_package(&test_app.group_repo, &vec![], &created_groups[0].id);

        util::create_permissions(&test_app.permission_repo, &test_app.group_repo, "task_package_task", &created_user.id, &AccessType::Write);

        let task = util::create_task_mc(&test_app.app_state.mongodb).await;

        let resp = test_app
        .call(
            &get_path(&created_groups[0].id, &task_package.id),
            SCOPE,
            add_tasks_to_package,
            test_app.invalid_authorizate(TestRequest::post().set_json(create_body(&task.id)), &created_user.id),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);
    }

    /// # Test: `test_add_task_to_task_package_no_permission`
    ///
    /// Validates the behavior of the `add_tasks_to_package` handler in Actix-Web when attempting to add tasks to a task package without the necessary permissions. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a test user (`created_user`) and an example group (`created_groups`) associated with `created_user`.
    ///    - Creates a task package (`task_package`) associated with the group.
    ///    - Does not grant the necessary permissions (`AccessType::Write`) to the user for adding tasks to the task package.
    ///    - Creates a test task (`task`).
    ///
    /// 2. Execution:
    ///    - Makes an API call to `add_tasks_to_package` using the test application instance, providing a valid authorization token for `created_user` and the task ID.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "FORBIDDEN" (403), indicating a lack of necessary permissions to add tasks to the task package.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_add_task_to_task_package_no_permission() {
        use crate::handlers::groups::group_id::task_packages::task_package_id::tasks::tasks::add_tasks_to_package;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_groups = util::create_example_groups(&test_app.group_repo, 1, created_user.id);

        let task_package = util::create_task_package(&test_app.group_repo, &vec![], &created_groups[0].id);

        let task = util::create_task_mc(&test_app.app_state.mongodb).await;

        let resp = test_app
        .call(
            &get_path(&created_groups[0].id, &task_package.id),
            SCOPE,
            add_tasks_to_package,
            test_app.valid_authorizate(TestRequest::post().set_json(create_body(&task.id)), &created_user.id),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::FORBIDDEN);
    }
}