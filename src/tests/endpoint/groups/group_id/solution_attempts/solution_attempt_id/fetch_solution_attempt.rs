//docu written with chat-gpt
#[cfg(test)]
mod fetch_solution_attempt_tests {
    use actix_web::{test::{TestRequest, self}, http};
    use serde_json::Value;
    use uuid::Uuid;

    use crate::{tests::{util::{self, create_other_test_user}, test::TestRepo}, models::{permissions::{NewRessource, OptionalUserAccessType}, groups::{NewUserPermission, CreateGroupResponse, GroupInfoResponse, GroupMetaDataResponse}, util::{AccessType, PagingSchema, OrderDir, Visibility, AnswerState}, task::{TaskPagingResponse, NewTempTask}, roles::{NewRole, UpdateRolePermission, NewRolePermission, NewRoleAccessType, UpdateRoleAccesType}, task_package::{TaskPackagesResponse, CreatedTaskPackageResponse}, solution_attempts::{CreatedSolutionAttemptResponse, SolutionAttemptWithAnswerListResponse}}};

    static SCOPE: &'static str = "/api/groups/{group_id}/solution_attempts/{solution_attempt_id}"; 
    
    fn get_path(group_id: &Uuid, solution_attempt_id: &Uuid) -> String {
        format!("/api/groups/{}/solution_attempts/{}/", group_id, solution_attempt_id)
    }

    /// # Test: `finish_soltuion_attempts`
    ///
    /// Validates the behavior of the `fetch_solution_attempt` handler in Actix-Web when attempting to fetch a solution attempt with read permission. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a test user (`created_user`).
    ///    - Creates an example group (`created_groups`) associated with `created_user`.
    ///    - Sets up permissions for reading the solution attempt (`solution_attempt` permission) for `created_user`.
    ///    - Creates a task package and a multiple-choice task.
    ///    - Creates a solution attempt by `created_user` for the task package.
    ///    - Sets up a correct answer for the solution attempt.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `fetch_solution_attempt` using the test application instance with a valid authorization token for `created_user`.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "OK" (200), indicating successful retrieval of the solution attempt.
    ///    - Asserts that the response contains the correct solution attempt details.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn finish_soltuion_attempts() {
        use crate::handlers::groups::group_id::solution_attempts::solution_attempt_id::solution_attempt_id::fetch_solution_attempt;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_groups = util::create_example_groups(&test_app.group_repo, 1, created_user.id);

        util::create_permissions(&test_app.permission_repo, &test_app.group_repo, "solution_attempt", &created_user.id, &AccessType::Read);

        let task_package = util::create_task_package(&test_app.group_repo, &vec![], &created_groups[0].id);

        let task = util::create_task_mc(&test_app.app_state.mongodb).await;

        let tasks = test_app.group_repo.add_tasks_to_package(&task_package.id, &created_groups[0].id, &vec![NewTempTask {
            task_doc_id: task.id.clone(),
            task_type: task.task_type.clone(),
        }]).unwrap();

        let solution_attempt = util::create_solution_attempt(&test_app.group_repo, &created_user.id, &task_package.id, &created_groups[0].id);

        util::create_correct_answer(&test_app.app_state.mongodb, &test_app.group_repo, &task, &solution_attempt.solution_list[0].answer_id).await;

        let resp = test_app
        .call(
            &get_path(&created_groups[0].id, &solution_attempt.solution_attempt.id),
            SCOPE,
            fetch_solution_attempt,
            test_app.valid_authorizate(TestRequest::get(), &created_user.id),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let response: SolutionAttemptWithAnswerListResponse = test::read_body_json(resp).await;

        assert_eq!(response.id, solution_attempt.solution_attempt.id);
        assert_eq!(response.task_package_id, task_package.id);
        assert_eq!(response.user_id, created_user.id);
        assert_eq!(response.visibility, solution_attempt.solution_attempt.visibility);
        assert_eq!(response.created_at, solution_attempt.solution_attempt.created_at.and_utc());
        assert_eq!(response.state, solution_attempt.state);
        //todo: assert answer_list
    }

    /// # Test: `finish_soltuion_attempts_wrong_group`
    ///
    /// Validates the behavior of the `fetch_solution_attempt` handler in Actix-Web when attempting to fetch a solution attempt with read permission for a solution attempt in a different group. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a test user (`created_user`).
    ///    - Creates an example group (`created_groups`) associated with `created_user`.
    ///    - Sets up permissions for reading the solution attempt (`solution_attempt` permission) for `created_user`.
    ///    - Creates a task package and a multiple-choice task.
    ///    - Creates a solution attempt by `created_user` for the task package.
    ///    - Sets up a correct answer for the solution attempt.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `fetch_solution_attempt` using the test application instance with a valid authorization token for `created_user` and a different group ID.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "NOT FOUND" (404), indicating that the solution attempt is not found in the specified group.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn finish_soltuion_attempts_wrong_group() {
        use crate::handlers::groups::group_id::solution_attempts::solution_attempt_id::solution_attempt_id::fetch_solution_attempt;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_groups = util::create_example_groups(&test_app.group_repo, 1, created_user.id);

        util::create_permissions(&test_app.permission_repo, &test_app.group_repo, "solution_attempt", &created_user.id, &AccessType::Read);

        let task_package = util::create_task_package(&test_app.group_repo, &vec![], &created_groups[0].id);

        let task = util::create_task_mc(&test_app.app_state.mongodb).await;

        let tasks = test_app.group_repo.add_tasks_to_package(&task_package.id, &created_groups[0].id, &vec![NewTempTask {
            task_doc_id: task.id.clone(),
            task_type: task.task_type.clone(),
        }]).unwrap();

        let solution_attempt = util::create_solution_attempt(&test_app.group_repo, &created_user.id, &task_package.id, &created_groups[0].id);

        util::create_correct_answer(&test_app.app_state.mongodb, &test_app.group_repo, &task, &solution_attempt.solution_list[0].answer_id).await;

        let resp = test_app
        .call(
            &get_path(&Uuid::new_v4(), &solution_attempt.solution_attempt.id),
            SCOPE,
            fetch_solution_attempt,
            test_app.valid_authorizate(TestRequest::get(), &created_user.id),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::NOT_FOUND);
    }

    /// # Test: `finish_soltuion_attempts_wrong_sollution_attempt`
    ///
    /// Validates the behavior of the `fetch_solution_attempt` handler in Actix-Web when attempting to fetch a solution attempt with read permission for a non-existent solution attempt ID. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a test user (`created_user`).
    ///    - Creates an example group (`created_groups`) associated with `created_user`.
    ///    - Sets up permissions for reading the solution attempt (`solution_attempt` permission) for `created_user`.
    ///    - Creates a task package and a multiple-choice task.
    ///    - Creates a solution attempt by `created_user` for the task package.
    ///    - Sets up a correct answer for the solution attempt.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `fetch_solution_attempt` using the test application instance with a valid authorization token for `created_user` and a non-existent solution attempt ID.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "NOT FOUND" (404), indicating that the specified solution attempt is not found.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn finish_soltuion_attempts_wrong_sollution_attempt() {
        use crate::handlers::groups::group_id::solution_attempts::solution_attempt_id::solution_attempt_id::fetch_solution_attempt;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_groups = util::create_example_groups(&test_app.group_repo, 1, created_user.id);

        util::create_permissions(&test_app.permission_repo, &test_app.group_repo, "solution_attempt", &created_user.id, &AccessType::Read);

        let task_package = util::create_task_package(&test_app.group_repo, &vec![], &created_groups[0].id);

        let task = util::create_task_mc(&test_app.app_state.mongodb).await;

        let tasks = test_app.group_repo.add_tasks_to_package(&task_package.id, &created_groups[0].id, &vec![NewTempTask {
            task_doc_id: task.id.clone(),
            task_type: task.task_type.clone(),
        }]).unwrap();

        let solution_attempt = util::create_solution_attempt(&test_app.group_repo, &created_user.id, &task_package.id, &created_groups[0].id);

        util::create_correct_answer(&test_app.app_state.mongodb, &test_app.group_repo, &task, &solution_attempt.solution_list[0].answer_id).await;

        let resp = test_app
        .call(
            &get_path(&created_groups[0].id, &Uuid::new_v4()),
            SCOPE,
            fetch_solution_attempt,
            test_app.valid_authorizate(TestRequest::get(), &created_user.id),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::NOT_FOUND);
    }

    /// # Test: `finish_soltuion_attempts_no_auth`
    ///
    /// Validates the behavior of the `fetch_solution_attempt` handler in Actix-Web when attempting to fetch a solution attempt without providing proper authentication. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a test user (`created_user`).
    ///    - Creates an example group (`created_groups`) associated with `created_user`.
    ///    - Sets up permissions for reading the solution attempt (`solution_attempt` permission) for `created_user`.
    ///    - Creates a task package and a multiple-choice task.
    ///    - Creates a solution attempt by `created_user` for the task package.
    ///    - Sets up a correct answer for the solution attempt.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `fetch_solution_attempt` using the test application instance with an invalid (unauthorized) authorization token for `created_user`.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "UNAUTHORIZED" (401), indicating that the request lacks proper authentication.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn finish_soltuion_attempts_no_auth() {
        use crate::handlers::groups::group_id::solution_attempts::solution_attempt_id::solution_attempt_id::fetch_solution_attempt;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_groups = util::create_example_groups(&test_app.group_repo, 1, created_user.id);

        util::create_permissions(&test_app.permission_repo, &test_app.group_repo, "solution_attempt", &created_user.id, &AccessType::Read);

        let task_package = util::create_task_package(&test_app.group_repo, &vec![], &created_groups[0].id);

        let task = util::create_task_mc(&test_app.app_state.mongodb).await;

        let tasks = test_app.group_repo.add_tasks_to_package(&task_package.id, &created_groups[0].id, &vec![NewTempTask {
            task_doc_id: task.id.clone(),
            task_type: task.task_type.clone(),
        }]).unwrap();

        let solution_attempt = util::create_solution_attempt(&test_app.group_repo, &created_user.id, &task_package.id, &created_groups[0].id);

        util::create_correct_answer(&test_app.app_state.mongodb, &test_app.group_repo, &task, &solution_attempt.solution_list[0].answer_id).await;

        let resp = test_app
        .call(
            &get_path(&created_groups[0].id, &solution_attempt.solution_attempt.id),
            SCOPE,
            fetch_solution_attempt,
            test_app.invalid_authorizate(TestRequest::get(), &created_user.id),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);
    }

    /// # Test: `finish_soltuion_attempts_no_permission`
    ///
    /// Validates the behavior of the `fetch_solution_attempt` handler in Actix-Web when attempting to fetch a solution attempt without the required permissions. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a test user (`created_user`).
    ///    - Creates an example group (`created_groups`) associated with `created_user`.
    ///    - Creates a task package and a multiple-choice task.
    ///    - Creates a solution attempt by `created_user` for the task package.
    ///    - Sets up a correct answer for the solution attempt.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `fetch_solution_attempt` using the test application instance with valid authentication for `created_user`.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "FORBIDDEN" (403), indicating that the request lacks the required permissions to fetch the solution attempt.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn finish_soltuion_attempts_no_permission() {
        use crate::handlers::groups::group_id::solution_attempts::solution_attempt_id::solution_attempt_id::fetch_solution_attempt;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_groups = util::create_example_groups(&test_app.group_repo, 1, created_user.id);

        let task_package = util::create_task_package(&test_app.group_repo, &vec![], &created_groups[0].id);

        let task = util::create_task_mc(&test_app.app_state.mongodb).await;

        let tasks = test_app.group_repo.add_tasks_to_package(&task_package.id, &created_groups[0].id, &vec![NewTempTask {
            task_doc_id: task.id.clone(),
            task_type: task.task_type.clone(),
        }]).unwrap();

        let solution_attempt = util::create_solution_attempt(&test_app.group_repo, &created_user.id, &task_package.id, &created_groups[0].id);

        util::create_correct_answer(&test_app.app_state.mongodb, &test_app.group_repo, &task, &solution_attempt.solution_list[0].answer_id).await;

        let resp = test_app
        .call(
            &get_path(&created_groups[0].id, &solution_attempt.solution_attempt.id),
            SCOPE,
            fetch_solution_attempt,
            test_app.valid_authorizate(TestRequest::get(), &created_user.id),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::FORBIDDEN);
    }

    /// # Test: `finish_soltuion_attempts_other`
    ///
    /// Validates the behavior of the `fetch_solution_attempt` handler in Actix-Web when attempting to fetch a solution attempt with the required permissions granted to another user. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates two test users (`created_user` and `me`).
    ///    - Creates an example group (`created_groups`) associated with `created_user`.
    ///    - Grants `me` the required permissions (`Read`) for fetching solution attempts.
    ///    - Creates a task package and a multiple-choice task.
    ///    - Creates a solution attempt by `created_user` for the task package.
    ///    - Sets up a correct answer for the solution attempt.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `fetch_solution_attempt` using the test application instance with valid authentication for `me`.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "OK" (200), indicating that the request is successful.
    ///    - Verifies that the response body contains the expected details of the fetched solution attempt.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn finish_soltuion_attempts_other() {
        use crate::handlers::groups::group_id::solution_attempts::solution_attempt_id::solution_attempt_id::fetch_solution_attempt;

        let test_app = TestRepo::new().await;

        let me = util::create_other_test_user(&test_app.user_repo);

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_groups = util::create_example_groups(&test_app.group_repo, 1, created_user.id);

        util::create_other_permissions(&test_app.permission_repo, &test_app.group_repo, "solution_attempt", &me.id, &AccessType::Read);

        let task_package = util::create_task_package(&test_app.group_repo, &vec![], &created_groups[0].id);

        let task = util::create_task_mc(&test_app.app_state.mongodb).await;

        let tasks = test_app.group_repo.add_tasks_to_package(&task_package.id, &created_groups[0].id, &vec![NewTempTask {
            task_doc_id: task.id.clone(),
            task_type: task.task_type.clone(),
        }]).unwrap();

        let solution_attempt = util::create_solution_attempt(&test_app.group_repo, &created_user.id, &task_package.id, &created_groups[0].id);

        util::create_correct_answer(&test_app.app_state.mongodb, &test_app.group_repo, &task, &solution_attempt.solution_list[0].answer_id).await;

        let resp = test_app
        .call(
            &get_path(&created_groups[0].id, &solution_attempt.solution_attempt.id),
            SCOPE,
            fetch_solution_attempt,
            test_app.valid_authorizate(TestRequest::get(), &me.id),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let response: SolutionAttemptWithAnswerListResponse = test::read_body_json(resp).await;

        assert_eq!(response.id, solution_attempt.solution_attempt.id);
        assert_eq!(response.task_package_id, task_package.id);
        assert_eq!(response.user_id, created_user.id);
        assert_eq!(response.visibility, solution_attempt.solution_attempt.visibility);
        assert_eq!(response.created_at, solution_attempt.solution_attempt.created_at.and_utc());
        assert_eq!(response.state, solution_attempt.state);
        //todo: assert answer_list
    }

    /// # Test: `finish_soltuion_attempts_other_private`
    ///
    /// Validates the behavior of the `fetch_solution_attempt` handler in Actix-Web when attempting to fetch a private solution attempt with the required permissions granted to another user. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates two test users (`created_user` and `me`).
    ///    - Creates an example group (`created_groups`) associated with `created_user`.
    ///    - Grants `me` the required permissions (`Read`) for fetching solution attempts.
    ///    - Creates a task package and a multiple-choice task.
    ///    - Creates a private solution attempt by `created_user` for the task package.
    ///    - Sets up a correct answer for the solution attempt.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `fetch_solution_attempt` using the test application instance with valid authentication for `me`.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "FORBIDDEN" (403), indicating that the request is denied due to insufficient permissions for accessing private solution attempts.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn finish_soltuion_attempts_other_private() {
        use crate::handlers::groups::group_id::solution_attempts::solution_attempt_id::solution_attempt_id::fetch_solution_attempt;

        let test_app = TestRepo::new().await;

        let me = util::create_other_test_user(&test_app.user_repo);

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_groups = util::create_example_groups(&test_app.group_repo, 1, created_user.id);

        util::create_other_permissions(&test_app.permission_repo, &test_app.group_repo, "solution_attempt", &me.id, &AccessType::Read);

        let task_package = util::create_task_package(&test_app.group_repo, &vec![], &created_groups[0].id);

        let task = util::create_task_mc(&test_app.app_state.mongodb).await;

        let tasks = test_app.group_repo.add_tasks_to_package(&task_package.id, &created_groups[0].id, &vec![NewTempTask {
            task_doc_id: task.id.clone(),
            task_type: task.task_type.clone(),
        }]).unwrap();

        let solution_attempt = util::create_private_solution_attempt(&test_app.group_repo, &created_user.id, &task_package.id, &created_groups[0].id);

        util::create_correct_answer(&test_app.app_state.mongodb, &test_app.group_repo, &task, &solution_attempt.solution_list[0].answer_id).await;

        let resp = test_app
        .call(
            &get_path(&created_groups[0].id, &solution_attempt.solution_attempt.id),
            SCOPE,
            fetch_solution_attempt,
            test_app.valid_authorizate(TestRequest::get(), &me.id),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::FORBIDDEN);
        //todo: assert answer_list
    }

    /// # Test: `finish_soltuion_attempts_other_no_permission`
    ///
    /// Validates the behavior of the `fetch_solution_attempt` handler in Actix-Web when attempting to fetch a solution attempt with the required permissions not granted to another user. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates two test users (`created_user` and `me`).
    ///    - Creates an example group (`created_groups`) associated with `created_user`.
    ///    - Grants `me` insufficient permissions for fetching solution attempts.
    ///    - Creates a task package and a multiple-choice task.
    ///    - Creates a solution attempt by `created_user` for the task package.
    ///    - Sets up a correct answer for the solution attempt.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `fetch_solution_attempt` using the test application instance with valid authentication for `me`.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "FORBIDDEN" (403), indicating that the request is denied due to insufficient permissions for accessing the solution attempt.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn finish_soltuion_attempts_other_no_permission() {
        use crate::handlers::groups::group_id::solution_attempts::solution_attempt_id::solution_attempt_id::fetch_solution_attempt;

        let test_app = TestRepo::new().await;

        let me = util::create_other_test_user(&test_app.user_repo);

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_groups = util::create_example_groups(&test_app.group_repo, 1, created_user.id);

        util::create_permissions(&test_app.permission_repo, &test_app.group_repo, "solution_attempt", &me.id, &AccessType::Read);

        let task_package = util::create_task_package(&test_app.group_repo, &vec![], &created_groups[0].id);

        let task = util::create_task_mc(&test_app.app_state.mongodb).await;

        let tasks = test_app.group_repo.add_tasks_to_package(&task_package.id, &created_groups[0].id, &vec![NewTempTask {
            task_doc_id: task.id.clone(),
            task_type: task.task_type.clone(),
        }]).unwrap();

        let solution_attempt = util::create_solution_attempt(&test_app.group_repo, &created_user.id, &task_package.id, &created_groups[0].id);

        util::create_correct_answer(&test_app.app_state.mongodb, &test_app.group_repo, &task, &solution_attempt.solution_list[0].answer_id).await;

        let resp = test_app
        .call(
            &get_path(&created_groups[0].id, &solution_attempt.solution_attempt.id),
            SCOPE,
            fetch_solution_attempt,
            test_app.valid_authorizate(TestRequest::get(), &me.id),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::FORBIDDEN);
        //todo: assert answer_list
    }
}