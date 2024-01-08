//docu written with chat-gpt
#[cfg(test)]
mod fetch_answer_tests {
    use actix_web::{test::{TestRequest, self}, http};
    use serde_json::Value;
    use uuid::Uuid;

    use crate::{tests::{util::{self, create_other_test_user}, test::TestRepo}, models::{permissions::{NewRessource, OptionalUserAccessType}, groups::{NewUserPermission, CreateGroupResponse, GroupInfoResponse, GroupMetaDataResponse}, util::{AccessType, PagingSchema, OrderDir, Visibility, AnswerState}, task::{TaskPagingResponse, NewTempTask}, roles::{NewRole, UpdateRolePermission, NewRolePermission, NewRoleAccessType, UpdateRoleAccesType}, task_package::{TaskPackagesResponse, CreatedTaskPackageResponse}, solution_attempts::{CreatedSolutionAttemptResponse, SolutionAttemptWithAnswerListResponse}, answer::AnswerResponse}};

    static SCOPE: &'static str = "/api/groups/{group_id}/answers/{answer_id}"; 
    
    fn get_path(group_id: &Uuid, answer_id: &Uuid) -> String {
        format!("/api/groups/{}/answers/{}/", group_id, answer_id)
    }

    /// # Test: `fetch_answer`
    ///
    /// Validates the behavior of the `fetch_answer` handler in Actix-Web when fetching a user answer with the necessary permissions. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a test user (`created_user`).
    ///    - Creates an example group (`created_groups`) associated with `created_user`.
    ///    - Grants `created_user` the required permissions for reading answers.
    ///    - Creates a task package, a multiple-choice task, and a solution attempt by `created_user`.
    ///    - Adds a correct answer for the solution attempt.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `fetch_answer` using the test application instance with valid authentication for `created_user`.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "OK" (200), indicating a successful request.
    ///    - Checks that the response body contains the expected details of the fetched answer.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn fetch_answer() {
        use crate::handlers::groups::group_id::answers::answer_id::answer_id::fetch_answer;
        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_groups = util::create_example_groups(&test_app.group_repo, 1, created_user.id);

        util::create_permissions(&test_app.permission_repo, &test_app.group_repo, "user_answer", &created_user.id, &AccessType::Read);

        let task_package = util::create_task_package(&test_app.group_repo, &vec![], &created_groups[0].id);

        let task = util::create_task_mc(&test_app.app_state.mongodb).await;

        let tasks = test_app.group_repo.add_tasks_to_package(&task_package.id, &created_groups[0].id, &vec![NewTempTask {
            task_doc_id: task.id.clone(),
            task_type: task.task_type.clone(),
        }]).unwrap();

        let solution_attempt = util::create_solution_attempt(&test_app.group_repo, &created_user.id, &task_package.id, &created_groups[0].id);

        let answer = util::create_correct_answer(&test_app.app_state.mongodb, &test_app.group_repo, &task, &solution_attempt.solution_list[0].answer_id).await;

        let resp = test_app
        .call(
            &get_path(&created_groups[0].id, &solution_attempt.solution_list[0].answer_id),
            SCOPE,
            fetch_answer,
            test_app.valid_authorizate(TestRequest::get(), &created_user.id),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let response: AnswerResponse = test::read_body_json(resp).await;



        assert_eq!(response.id, solution_attempt.solution_list[0].answer_id);
        assert_eq!(response.correct, false);
        assert_eq!(response.state, AnswerState::Active);
        assert_eq!(response.task_id, tasks[0].id);
        assert_eq!(response.created_from, created_user.id);
        assert_eq!(response.answer_doc.as_ref().unwrap(), &answer.solution);
    }

    /// # Test: `fetch_answer_other`
    ///
    /// Validates the behavior of the `fetch_answer` handler in Actix-Web when attempting to fetch a user answer with the necessary permissions on behalf of another user (`me`). The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates two test users (`created_user` and `me`).
    ///    - Creates an example group (`created_groups`) associated with `created_user`.
    ///    - Grants `me` the required permissions for reading answers.
    ///    - Creates a task package, a multiple-choice task, and a solution attempt by `created_user`.
    ///    - Adds a correct answer for the solution attempt.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `fetch_answer` using the test application instance with valid authentication for `me`.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "OK" (200), indicating a successful request.
    ///    - Checks that the response body contains the expected details of the fetched answer.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn fetch_answer_other() {
        use crate::handlers::groups::group_id::answers::answer_id::answer_id::fetch_answer;
        let test_app = TestRepo::new().await;

        let me = util::create_other_test_user(&test_app.user_repo);
        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_groups = util::create_example_groups(&test_app.group_repo, 1, created_user.id);

        util::create_other_permissions(&test_app.permission_repo, &test_app.group_repo, "user_answer", &me.id, &AccessType::Read);

        let task_package = util::create_task_package(&test_app.group_repo, &vec![], &created_groups[0].id);

        let task = util::create_task_mc(&test_app.app_state.mongodb).await;

        let tasks = test_app.group_repo.add_tasks_to_package(&task_package.id, &created_groups[0].id, &vec![NewTempTask {
            task_doc_id: task.id.clone(),
            task_type: task.task_type.clone(),
        }]).unwrap();

        let solution_attempt = util::create_solution_attempt(&test_app.group_repo, &created_user.id, &task_package.id, &created_groups[0].id);

        let answer = util::create_correct_answer(&test_app.app_state.mongodb, &test_app.group_repo, &task, &solution_attempt.solution_list[0].answer_id).await;

        let resp = test_app
        .call(
            &get_path(&created_groups[0].id, &solution_attempt.solution_list[0].answer_id),
            SCOPE,
            fetch_answer,
            test_app.valid_authorizate(TestRequest::get(), &me.id),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let response: AnswerResponse = test::read_body_json(resp).await;

        assert_eq!(response.id, solution_attempt.solution_list[0].answer_id);
        assert_eq!(response.correct, false);
        assert_eq!(response.state, AnswerState::Active);
        assert_eq!(response.task_id, tasks[0].id);
        assert_eq!(response.created_from, created_user.id);
        assert_eq!(response.answer_doc.as_ref().unwrap(), &answer.solution);
    }

    #[actix_web::test]
    #[serial_test::serial]
    async fn fetch_answer_other_no_permission() {
        use crate::handlers::groups::group_id::answers::answer_id::answer_id::fetch_answer;
        let test_app = TestRepo::new().await;

        let me = util::create_other_test_user(&test_app.user_repo);
        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_groups = util::create_example_groups(&test_app.group_repo, 1, created_user.id);

        util::create_permissions(&test_app.permission_repo, &test_app.group_repo, "user_answer", &me.id, &AccessType::Read);

        let task_package = util::create_task_package(&test_app.group_repo, &vec![], &created_groups[0].id);

        let task = util::create_task_mc(&test_app.app_state.mongodb).await;

        let tasks = test_app.group_repo.add_tasks_to_package(&task_package.id, &created_groups[0].id, &vec![NewTempTask {
            task_doc_id: task.id.clone(),
            task_type: task.task_type.clone(),
        }]).unwrap();

        let solution_attempt = util::create_solution_attempt(&test_app.group_repo, &created_user.id, &task_package.id, &created_groups[0].id);

        let answer = util::create_correct_answer(&test_app.app_state.mongodb, &test_app.group_repo, &task, &solution_attempt.solution_list[0].answer_id).await;

        let resp = test_app
        .call(
            &get_path(&created_groups[0].id, &solution_attempt.solution_list[0].answer_id),
            SCOPE,
            fetch_answer,
            test_app.valid_authorizate(TestRequest::get(), &me.id),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::FORBIDDEN);
    }

    /// # Test: `fetch_answer_other_no_permission`
    ///
    /// Validates the behavior of the `fetch_answer` handler in Actix-Web when attempting to fetch a user answer with insufficient permissions on behalf of another user (`me`). The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates two test users (`created_user` and `me`).
    ///    - Creates an example group (`created_groups`) associated with `created_user`.
    ///    - Grants `me` limited permissions for reading answers.
    ///    - Creates a task package, a multiple-choice task, and a solution attempt by `created_user`.
    ///    - Adds a correct answer for the solution attempt.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `fetch_answer` using the test application instance with valid authentication for `me`.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "FORBIDDEN" (403), indicating insufficient permissions.
    ///    - Checks that the response body does not reveal details of the fetched answer.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn fetch_answer_no_auth() {
        use crate::handlers::groups::group_id::answers::answer_id::answer_id::fetch_answer;
        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_groups = util::create_example_groups(&test_app.group_repo, 1, created_user.id);

        util::create_permissions(&test_app.permission_repo, &test_app.group_repo, "user_answer", &created_user.id, &AccessType::Read);

        let task_package = util::create_task_package(&test_app.group_repo, &vec![], &created_groups[0].id);

        let task = util::create_task_mc(&test_app.app_state.mongodb).await;

        let tasks = test_app.group_repo.add_tasks_to_package(&task_package.id, &created_groups[0].id, &vec![NewTempTask {
            task_doc_id: task.id.clone(),
            task_type: task.task_type.clone(),
        }]).unwrap();

        let solution_attempt = util::create_solution_attempt(&test_app.group_repo, &created_user.id, &task_package.id, &created_groups[0].id);

        let answer = util::create_correct_answer(&test_app.app_state.mongodb, &test_app.group_repo, &task, &solution_attempt.solution_list[0].answer_id).await;

        let resp = test_app
        .call(
            &get_path(&created_groups[0].id, &solution_attempt.solution_list[0].answer_id),
            SCOPE,
            fetch_answer,
            test_app.invalid_authorizate(TestRequest::get(), &created_user.id),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);
    }

    /// # Test: `fetch_answer_wrong_answer_id`
    ///
    /// Validates the behavior of the `fetch_answer` handler in Actix-Web when attempting to fetch a user answer with an invalid answer ID. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a test user (`created_user`).
    ///    - Creates an example group (`created_groups`) associated with `created_user`.
    ///    - Grants `created_user` permissions for reading answers.
    ///    - Creates a task package, a multiple-choice task, and a solution attempt by `created_user`.
    ///    - Adds a correct answer for the solution attempt.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `fetch_answer` using the test application instance with a non-existent answer ID.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "NOT_FOUND" (404), indicating that the answer ID is invalid.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn fetch_answer_wrong_answer_id() {
        use crate::handlers::groups::group_id::answers::answer_id::answer_id::fetch_answer;
        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_groups = util::create_example_groups(&test_app.group_repo, 1, created_user.id);

        util::create_permissions(&test_app.permission_repo, &test_app.group_repo, "user_answer", &created_user.id, &AccessType::Read);

        let task_package = util::create_task_package(&test_app.group_repo, &vec![], &created_groups[0].id);

        let task = util::create_task_mc(&test_app.app_state.mongodb).await;

        let tasks = test_app.group_repo.add_tasks_to_package(&task_package.id, &created_groups[0].id, &vec![NewTempTask {
            task_doc_id: task.id.clone(),
            task_type: task.task_type.clone(),
        }]).unwrap();

        let solution_attempt = util::create_solution_attempt(&test_app.group_repo, &created_user.id, &task_package.id, &created_groups[0].id);

        let answer = util::create_correct_answer(&test_app.app_state.mongodb, &test_app.group_repo, &task, &solution_attempt.solution_list[0].answer_id).await;

        let resp = test_app
        .call(
            &get_path(&created_groups[0].id, &Uuid::new_v4()),
            SCOPE,
            fetch_answer,
            test_app.valid_authorizate(TestRequest::get(), &created_user.id),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::NOT_FOUND);
    }

    /// # Test: `fetch_answer_wrong_group_id`
    ///
    /// Validates the behavior of the `fetch_answer` handler in Actix-Web when attempting to fetch a user answer with an invalid group ID. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a test user (`created_user`).
    ///    - Creates an example group (`created_groups`) associated with `created_user`.
    ///    - Grants `created_user` permissions for reading answers.
    ///    - Creates a task package, a multiple-choice task, and a solution attempt by `created_user`.
    ///    - Adds a correct answer for the solution attempt.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `fetch_answer` using the test application instance with a non-existent group ID.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "NOT_FOUND" (404), indicating that the group ID is invalid.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn fetch_answer_wrong_group_id() {
        use crate::handlers::groups::group_id::answers::answer_id::answer_id::fetch_answer;
        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_groups = util::create_example_groups(&test_app.group_repo, 1, created_user.id);

        util::create_permissions(&test_app.permission_repo, &test_app.group_repo, "user_answer", &created_user.id, &AccessType::Read);

        let task_package = util::create_task_package(&test_app.group_repo, &vec![], &created_groups[0].id);

        let task = util::create_task_mc(&test_app.app_state.mongodb).await;

        let tasks = test_app.group_repo.add_tasks_to_package(&task_package.id, &created_groups[0].id, &vec![NewTempTask {
            task_doc_id: task.id.clone(),
            task_type: task.task_type.clone(),
        }]).unwrap();

        let solution_attempt = util::create_solution_attempt(&test_app.group_repo, &created_user.id, &task_package.id, &created_groups[0].id);

        let answer = util::create_correct_answer(&test_app.app_state.mongodb, &test_app.group_repo, &task, &solution_attempt.solution_list[0].answer_id).await;

        let resp = test_app
        .call(
            &get_path(&Uuid::new_v4(), &solution_attempt.solution_list[0].answer_id),
            SCOPE,
            fetch_answer,
            test_app.valid_authorizate(TestRequest::get(), &created_user.id),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::NOT_FOUND);
    }

    /// # Test: `fetch_answer_no_permission`
    ///
    /// Validates the behavior of the `fetch_answer` handler in Actix-Web when attempting to fetch a user answer without the required permissions. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a test user (`created_user`).
    ///    - Creates an example group (`created_groups`) associated with `created_user`.
    ///    - Does not grant `created_user` permissions for reading answers.
    ///    - Creates a task package, a multiple-choice task, and a solution attempt by `created_user`.
    ///    - Adds a correct answer for the solution attempt.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `fetch_answer` using the test application instance with the valid group ID and answer ID.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "FORBIDDEN" (403), indicating that the user does not have the required permissions.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn fetch_answer_no_permission() {
        use crate::handlers::groups::group_id::answers::answer_id::answer_id::fetch_answer;
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

        let answer = util::create_correct_answer(&test_app.app_state.mongodb, &test_app.group_repo, &task, &solution_attempt.solution_list[0].answer_id).await;

        let resp = test_app
        .call(
            &get_path(&created_groups[0].id, &solution_attempt.solution_list[0].answer_id),
            SCOPE,
            fetch_answer,
            test_app.valid_authorizate(TestRequest::get(), &created_user.id),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::FORBIDDEN);
    }
}