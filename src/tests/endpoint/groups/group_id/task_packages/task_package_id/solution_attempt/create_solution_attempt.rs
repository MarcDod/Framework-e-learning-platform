//docu written with chat-gpt
#[cfg(test)]
mod create_solution_attempt_tests {
    use actix_web::{test::{TestRequest, self}, http};
    use serde_json::Value;
    use uuid::Uuid;

    use crate::{tests::{util, test::TestRepo}, models::{permissions::{NewRessource, OptionalUserAccessType}, groups::{NewUserPermission, CreateGroupResponse, GroupInfoResponse, GroupMetaDataResponse}, util::{AccessType, PagingSchema, OrderDir, Visibility}, task::{TaskPagingResponse, NewTempTask}, roles::{NewRole, UpdateRolePermission, NewRolePermission, NewRoleAccessType, UpdateRoleAccesType}, task_package::{TaskPackagesResponse, CreatedTaskPackageResponse}, solution_attempts::CreatedSolutionAttemptResponse}};

    static SCOPE: &'static str = "/api/groups/{group_id}/task_packages/{task_package_id}/solution_attempts"; 
    
    fn get_path(group_id: &Uuid, task_package_id: &Uuid) -> String {
        format!("/api/groups/{}/task_packages/{}/solution_attempts/", group_id, task_package_id)
    }

    fn create_body() -> Value {
        serde_json::json!({})
    }

    /// # Test: `create_soltuion_attempts`
    ///
    /// Validates the behavior of the `create_solution_attempt` handler in Actix-Web when creating a solution attempt for a task package. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a test user (`created_user`) and an example group (`created_groups`) associated with `created_user`.
    ///    - Grants the necessary permissions (`AccessType::Create`) to the user for creating solution attempts.
    ///    - Creates a task package (`task_package`) associated with the group.
    ///    - Adds a task (`task`) to the task package.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `create_solution_attempt` using the test application instance, providing a valid authorization token for `created_user`.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "CREATED" (201), indicating a successful creation of the solution attempt.
    ///    - Validates the structure and content of the response, including task package ID, user ID, visibility, and the answer list.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn create_soltuion_attempts() {
        use crate::handlers::groups::group_id::task_packages::task_package_id::solution_attempts::solution_attempts::create_solution_attempt;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_groups = util::create_example_groups(&test_app.group_repo, 1, created_user.id);

        util::create_permissions(&test_app.permission_repo, &test_app.group_repo, "solution_attempt", &created_user.id, &AccessType::Create);

        let task_package = util::create_task_package(&test_app.group_repo, &vec![], &created_groups[0].id);

        let task = util::create_task_mc(&test_app.app_state.mongodb).await;

        let tasks = test_app.group_repo.add_tasks_to_package(&task_package.id, &created_groups[0].id, &vec![NewTempTask {
            task_doc_id: task.id.clone(),
            task_type: task.task_type.clone(),
        }]).unwrap();

        let resp = test_app
        .call(
            &get_path(&created_groups[0].id, &task_package.id),
            SCOPE,
            create_solution_attempt,
            test_app.valid_authorizate(TestRequest::post().set_json(create_body()), &created_user.id),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::CREATED);

        let response: CreatedSolutionAttemptResponse = test::read_body_json(resp).await;

        assert_eq!(response.task_package_id, task_package.id);
        assert_eq!(response.user_id, created_user.id);
        assert_eq!(response.visibility, Visibility::Public);
        assert_eq!(response.answer_list.len(), 1);
        assert_eq!(response.answer_list[0].answer_doc_id, Uuid::default());
        assert_eq!(response.answer_list[0].task_doc_id, task.id);
        assert_eq!(response.answer_list[0].task_type, task.task_type);
        assert_eq!(response.answer_list[0].task_id, tasks[0].id);
        // TODO: assert other
    }

    /// # Test: `create_soltuion_attempts_wrong_group`
    ///
    /// Validates the behavior of the `create_solution_attempt` handler in Actix-Web when attempting to create a solution attempt for a task package associated with a non-existent group. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a test user (`created_user`) and an example group (`created_groups`) associated with `created_user`.
    ///    - Grants the necessary permissions (`AccessType::Create`) to the user for creating solution attempts.
    ///    - Creates a task package (`task_package`) associated with the group.
    ///    - Adds a task (`task`) to the task package.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `create_solution_attempt` using the test application instance, providing a valid authorization token for `created_user`. The group ID in the path is replaced with a non-existent UUID.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "NOT_FOUND" (404), indicating that the specified group was not found.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn create_soltuion_attempts_wrong_group() {
        use crate::handlers::groups::group_id::task_packages::task_package_id::solution_attempts::solution_attempts::create_solution_attempt;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_groups = util::create_example_groups(&test_app.group_repo, 1, created_user.id);

        util::create_permissions(&test_app.permission_repo, &test_app.group_repo, "solution_attempt", &created_user.id, &AccessType::Create);

        let task_package = util::create_task_package(&test_app.group_repo, &vec![], &created_groups[0].id);

        let task = util::create_task_mc(&test_app.app_state.mongodb).await;

        let tasks = test_app.group_repo.add_tasks_to_package(&task_package.id, &created_groups[0].id, &vec![NewTempTask {
            task_doc_id: task.id.clone(),
            task_type: task.task_type.clone(),
        }]).unwrap();

        let resp = test_app
        .call(
            &get_path(&Uuid::new_v4(), &task_package.id),
            SCOPE,
            create_solution_attempt,
            test_app.valid_authorizate(TestRequest::post().set_json(create_body()), &created_user.id),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::NOT_FOUND);
    }

    /// # Test: `create_soltuion_attempts_wrong_task_package`
    ///
    /// Validates the behavior of the `create_solution_attempt` handler in Actix-Web when attempting to create a solution attempt for a non-existent task package associated with a group. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a test user (`created_user`) and an example group (`created_groups`) associated with `created_user`.
    ///    - Grants the necessary permissions (`AccessType::Create`) to the user for creating solution attempts.
    ///    - Creates a task package (`task_package`) associated with the group.
    ///    - Adds a task (`task`) to the task package.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `create_solution_attempt` using the test application instance, providing a valid authorization token for `created_user`. The task package ID in the path is replaced with a non-existent UUID.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "NOT_FOUND" (404), indicating that the specified task package was not found.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn create_soltuion_attempts_wrong_task_package() {
        use crate::handlers::groups::group_id::task_packages::task_package_id::solution_attempts::solution_attempts::create_solution_attempt;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_groups = util::create_example_groups(&test_app.group_repo, 1, created_user.id);

        util::create_permissions(&test_app.permission_repo, &test_app.group_repo, "solution_attempt", &created_user.id, &AccessType::Create);

        let task_package = util::create_task_package(&test_app.group_repo, &vec![], &created_groups[0].id);

        let task = util::create_task_mc(&test_app.app_state.mongodb).await;

        let tasks = test_app.group_repo.add_tasks_to_package(&task_package.id, &created_groups[0].id, &vec![NewTempTask {
            task_doc_id: task.id.clone(),
            task_type: task.task_type.clone(),
        }]).unwrap();

        let resp = test_app
        .call(
            &get_path(&created_groups[0].id, &Uuid::new_v4()),
            SCOPE,
            create_solution_attempt,
            test_app.valid_authorizate(TestRequest::post().set_json(create_body()), &created_user.id),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::NOT_FOUND);
    }

    /// # Test: `create_soltuion_attempts_no_auth`
    ///
    /// Validates the behavior of the `create_solution_attempt` handler in Actix-Web when attempting to create a solution attempt without proper authentication. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a test user (`created_user`) and an example group (`created_groups`) associated with `created_user`.
    ///    - Grants the necessary permissions (`AccessType::Create`) to the user for creating solution attempts.
    ///    - Creates a task package (`task_package`) associated with the group.
    ///    - Adds a task (`task`) to the task package.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `create_solution_attempt` using the test application instance without providing a valid authorization token.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "UNAUTHORIZED" (401), indicating that the request lacks proper authentication.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn create_soltuion_attempts_no_auth() {
        use crate::handlers::groups::group_id::task_packages::task_package_id::solution_attempts::solution_attempts::create_solution_attempt;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_groups = util::create_example_groups(&test_app.group_repo, 1, created_user.id);

        util::create_permissions(&test_app.permission_repo, &test_app.group_repo, "solution_attempt", &created_user.id, &AccessType::Create);

        let task_package = util::create_task_package(&test_app.group_repo, &vec![], &created_groups[0].id);

        let task = util::create_task_mc(&test_app.app_state.mongodb).await;

        let tasks = test_app.group_repo.add_tasks_to_package(&task_package.id, &created_groups[0].id, &vec![NewTempTask {
            task_doc_id: task.id.clone(),
            task_type: task.task_type.clone(),
        }]).unwrap();

        let resp = test_app
        .call(
            &get_path(&created_groups[0].id, &task_package.id),
            SCOPE,
            create_solution_attempt,
            test_app.invalid_authorizate(TestRequest::post().set_json(create_body()), &created_user.id),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);
    }

    /// # Test: `create_soltuion_attempts_no_permission`
    ///
    /// Validates the behavior of the `create_solution_attempt` handler in Actix-Web when attempting to create a solution attempt without proper permissions. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a test user (`created_user`) and an example group (`created_groups`) associated with `created_user`.
    ///    - Creates a task package (`task_package`) associated with the group.
    ///    - Adds a task (`task`) to the task package.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `create_solution_attempt` using the test application instance with a valid authorization token for `created_user`.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "FORBIDDEN" (403), indicating that the user lacks the necessary permissions to create a solution attempt.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn create_soltuion_attempts_no_permission() {
        use crate::handlers::groups::group_id::task_packages::task_package_id::solution_attempts::solution_attempts::create_solution_attempt;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_groups = util::create_example_groups(&test_app.group_repo, 1, created_user.id);

        let task_package = util::create_task_package(&test_app.group_repo, &vec![], &created_groups[0].id);

        let task = util::create_task_mc(&test_app.app_state.mongodb).await;

        let tasks = test_app.group_repo.add_tasks_to_package(&task_package.id, &created_groups[0].id, &vec![NewTempTask {
            task_doc_id: task.id.clone(),
            task_type: task.task_type.clone(),
        }]).unwrap();

        let resp = test_app
        .call(
            &get_path(&created_groups[0].id, &task_package.id),
            SCOPE,
            create_solution_attempt,
            test_app.valid_authorizate(TestRequest::post().set_json(create_body()), &created_user.id),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::FORBIDDEN);
    }
}