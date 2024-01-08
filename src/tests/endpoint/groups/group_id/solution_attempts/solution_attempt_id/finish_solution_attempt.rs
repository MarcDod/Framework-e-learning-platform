//docu written with chat-gpt
#[cfg(test)]
mod finish_solution_attempt_tests {
    use actix_web::{test::{TestRequest, self}, http};
    use serde_json::Value;
    use uuid::Uuid;

    use crate::{tests::{util, test::TestRepo}, models::{permissions::{NewRessource, OptionalUserAccessType}, groups::{NewUserPermission, CreateGroupResponse, GroupInfoResponse, GroupMetaDataResponse}, util::{AccessType, PagingSchema, OrderDir, Visibility, AnswerState}, task::{TaskPagingResponse, NewTempTask}, roles::{NewRole, UpdateRolePermission, NewRolePermission, NewRoleAccessType, UpdateRoleAccesType}, task_package::{TaskPackagesResponse, CreatedTaskPackageResponse}, solution_attempts::CreatedSolutionAttemptResponse}};

    static SCOPE: &'static str = "/api/groups/{group_id}/solution_attempts/{solution_attempt_id}"; 
    
    fn get_path(group_id: &Uuid, solution_attempt_id: &Uuid) -> String {
        format!("/api/groups/{}/solution_attempts/{}/finish", group_id, solution_attempt_id)
    }

    /// # Test: `finish_soltuion_attempts`
    ///
    /// Validates the behavior of the `finish_solution_attempt` handler in Actix-Web when attempting to finish a solution attempt with proper permissions. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a test user (`created_user`) and an example group (`created_groups`) associated with `created_user`.
    ///    - Creates a task package (`task_package`) associated with the group.
    ///    - Adds a task (`task`) to the task package.
    ///    - Creates a solution attempt (`solution_attempt`) associated with the user, task package, and group.
    ///    - Adds a correct answer to the solution attempt.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `finish_solution_attempt` using the test application instance with a valid authorization token for `created_user`.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "NO_CONTENT" (204), indicating that the solution attempt was successfully finished.
    ///    - Checks that the associated answers in the database have the correct state and correctness status.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn finish_soltuion_attempts() {
        use crate::handlers::groups::group_id::solution_attempts::solution_attempt_id::solution_attempt_id::finish_solution_attempt;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_groups = util::create_example_groups(&test_app.group_repo, 1, created_user.id);

        util::create_permissions(&test_app.permission_repo, &test_app.group_repo, "solution_attempt_finish", &created_user.id, &AccessType::Write);

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
            finish_solution_attempt,
            test_app.valid_authorizate(TestRequest::post(), &created_user.id),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::NO_CONTENT);

        let answers = test_app.group_repo.fetch_answers_from_solution_attempt(&solution_attempt.solution_attempt.id).unwrap();
    
        assert_eq!(answers.len(), 1);
        assert_eq!(answers[0].state, AnswerState::Done);
        assert_eq!(answers[0].correct, true);
    }

    /// # Test: `finish_soltuion_attempts_wrong_answer`
    ///
    /// Validates the behavior of the `finish_solution_attempt` handler in Actix-Web when attempting to finish a solution attempt with a wrong answer. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a test user (`created_user`) and an example group (`created_groups`) associated with `created_user`.
    ///    - Creates a task package (`task_package`) associated with the group.
    ///    - Adds a task (`task`) to the task package.
    ///    - Creates a solution attempt (`solution_attempt`) associated with the user, task package, and group.
    ///    - Adds an incorrect answer to the solution attempt.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `finish_solution_attempt` using the test application instance with a valid authorization token for `created_user`.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "NO_CONTENT" (204), indicating that the solution attempt was successfully finished.
    ///    - Checks that the associated answers in the database have the correct state and correctness status (incorrect).
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn finish_soltuion_attempts_wrong_answer() {
        use crate::handlers::groups::group_id::solution_attempts::solution_attempt_id::solution_attempt_id::finish_solution_attempt;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_groups = util::create_example_groups(&test_app.group_repo, 1, created_user.id);

        util::create_permissions(&test_app.permission_repo, &test_app.group_repo, "solution_attempt_finish", &created_user.id, &AccessType::Write);

        let task_package = util::create_task_package(&test_app.group_repo, &vec![], &created_groups[0].id);

        let task = util::create_task_mc(&test_app.app_state.mongodb).await;

        let tasks = test_app.group_repo.add_tasks_to_package(&task_package.id, &created_groups[0].id, &vec![NewTempTask {
            task_doc_id: task.id.clone(),
            task_type: task.task_type.clone(),
        }]).unwrap();

        let solution_attempt = util::create_solution_attempt(&test_app.group_repo, &created_user.id, &task_package.id, &created_groups[0].id);

        util::create_incorrect_answer(&test_app.app_state.mongodb, &test_app.group_repo, &solution_attempt.solution_list[0].answer_id).await;

        let resp = test_app
        .call(
            &get_path(&created_groups[0].id, &solution_attempt.solution_attempt.id),
            SCOPE,
            finish_solution_attempt,
            test_app.valid_authorizate(TestRequest::post(), &created_user.id),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::NO_CONTENT);

        let answers = test_app.group_repo.fetch_answers_from_solution_attempt(&solution_attempt.solution_attempt.id).unwrap();
    
        assert_eq!(answers.len(), 1);
        assert_eq!(answers[0].state, AnswerState::Done);
        assert_eq!(answers[0].correct, false);
    }

    // # Test: `finish_soltuion_attempts_other`
    ///
    /// Validates the behavior of the `finish_solution_attempt` handler in Actix-Web when a user with write permission other than the creator attempts to finish a solution attempt. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates two test users (`me` and `created_user`) and an example group (`created_groups`) associated with `created_user`.
    ///    - Creates a task package (`task_package`) associated with the group.
    ///    - Adds a task (`task`) to the task package.
    ///    - Creates a solution attempt (`solution_attempt`) associated with `created_user`, the task package, and the group.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `finish_solution_attempt` using the test application instance with a valid authorization token for `me`.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "NO_CONTENT" (204), indicating that the solution attempt was successfully finished.
    ///    - Checks that the associated answers in the database have the correct state (Done).
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn finish_soltuion_attempts_other() {
        use crate::handlers::groups::group_id::solution_attempts::solution_attempt_id::solution_attempt_id::finish_solution_attempt;

        let test_app = TestRepo::new().await;

        let me = util::create_other_test_user(&test_app.user_repo);

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_groups = util::create_example_groups(&test_app.group_repo, 1, created_user.id);

        util::create_other_permissions(&test_app.permission_repo, &test_app.group_repo, "solution_attempt_finish", &me.id, &AccessType::Write);

        let task_package = util::create_task_package(&test_app.group_repo, &vec![], &created_groups[0].id);

        let task = util::create_task_mc(&test_app.app_state.mongodb).await;

        let tasks = test_app.group_repo.add_tasks_to_package(&task_package.id, &created_groups[0].id, &vec![NewTempTask {
            task_doc_id: task.id.clone(),
            task_type: task.task_type.clone(),
        }]).unwrap();

        let solution_attempt = util::create_solution_attempt(&test_app.group_repo, &created_user.id, &task_package.id, &created_groups[0].id);

        let resp = test_app
        .call(
            &get_path(&created_groups[0].id, &solution_attempt.solution_attempt.id),
            SCOPE,
            finish_solution_attempt,
            test_app.valid_authorizate(TestRequest::post(), &me.id),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::NO_CONTENT);

        let answers = test_app.group_repo.fetch_answers_from_solution_attempt(&solution_attempt.solution_attempt.id).unwrap();
    
        assert_eq!(answers.len(), 1);
        assert_eq!(answers[0].state, AnswerState::Done);
    }

    /// # Test: `finish_soltuion_attempts_other_private`
    ///
    /// Validates the behavior of the `finish_solution_attempt` handler in Actix-Web when a user with write permission other than the creator attempts to finish a private solution attempt. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates two test users (`me` and `created_user`) and an example group (`created_groups`) associated with `created_user`.
    ///    - Creates a task package (`task_package`) associated with the group.
    ///    - Adds a task (`task`) to the task package.
    ///    - Creates a private solution attempt (`solution_attempt`) associated with `created_user`, the task package, and the group.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `finish_solution_attempt` using the test application instance with a valid authorization token for `me`.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "FORBIDDEN" (403), indicating that finishing a private solution attempt is not allowed for users other than the creator.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn finish_soltuion_attempts_other_private() {
        use crate::handlers::groups::group_id::solution_attempts::solution_attempt_id::solution_attempt_id::finish_solution_attempt;

        let test_app = TestRepo::new().await;

        let me = util::create_other_test_user(&test_app.user_repo);

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_groups = util::create_example_groups(&test_app.group_repo, 1, created_user.id);

        util::create_other_permissions(&test_app.permission_repo, &test_app.group_repo, "solution_attempt_finish", &me.id, &AccessType::Write);

        let task_package = util::create_task_package(&test_app.group_repo, &vec![], &created_groups[0].id);

        let task = util::create_task_mc(&test_app.app_state.mongodb).await;

        let tasks = test_app.group_repo.add_tasks_to_package(&task_package.id, &created_groups[0].id, &vec![NewTempTask {
            task_doc_id: task.id.clone(),
            task_type: task.task_type.clone(),
        }]).unwrap();

        let solution_attempt = util::create_private_solution_attempt(&test_app.group_repo, &created_user.id, &task_package.id, &created_groups[0].id);

        let resp = test_app
        .call(
            &get_path(&created_groups[0].id, &solution_attempt.solution_attempt.id),
            SCOPE,
            finish_solution_attempt,
            test_app.valid_authorizate(TestRequest::post(), &me.id),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::FORBIDDEN);
    }

    /// # Test: `finish_soltuion_attempts_no_permission`
    ///
    /// Validates the behavior of the `finish_solution_attempt` handler in Actix-Web when a user without write permission attempts to finish a solution attempt. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a test user (`created_user`) and an example group (`created_groups`) associated with `created_user`.
    ///    - Creates a task package (`task_package`) associated with the group.
    ///    - Adds a task (`task`) to the task package.
    ///    - Creates a solution attempt (`solution_attempt`) associated with `created_user`, the task package, and the group.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `finish_solution_attempt` using the test application instance with a valid authorization token for `created_user`.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "FORBIDDEN" (403), indicating that finishing a solution attempt is not allowed without the necessary permissions.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn finish_soltuion_attempts_no_permission() {
        use crate::handlers::groups::group_id::solution_attempts::solution_attempt_id::solution_attempt_id::finish_solution_attempt;

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

        let resp = test_app
        .call(
            &get_path(&created_groups[0].id, &solution_attempt.solution_attempt.id),
            SCOPE,
            finish_solution_attempt,
            test_app.valid_authorizate(TestRequest::post(), &created_user.id),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::FORBIDDEN);
    }

    /// # Test: `finish_soltuion_attempts_no_auth`
    ///
    /// Validates the behavior of the `finish_solution_attempt` handler in Actix-Web when an unauthenticated user attempts to finish a solution attempt. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a test user (`created_user`) and an example group (`created_groups`) associated with `created_user`.
    ///    - Creates a task package (`task_package`) associated with the group.
    ///    - Adds a task (`task`) to the task package.
    ///    - Creates a solution attempt (`solution_attempt`) associated with `created_user`, the task package, and the group.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `finish_solution_attempt` using the test application instance without providing a valid authorization token.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "UNAUTHORIZED" (401), indicating that finishing a solution attempt is not allowed without authentication.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn finish_soltuion_attempts_no_auth() {
        use crate::handlers::groups::group_id::solution_attempts::solution_attempt_id::solution_attempt_id::finish_solution_attempt;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_groups = util::create_example_groups(&test_app.group_repo, 1, created_user.id);

        util::create_permissions(&test_app.permission_repo, &test_app.group_repo, "solution_attempt_finish", &created_user.id, &AccessType::Write);

        let task_package = util::create_task_package(&test_app.group_repo, &vec![], &created_groups[0].id);

        let task = util::create_task_mc(&test_app.app_state.mongodb).await;

        let tasks = test_app.group_repo.add_tasks_to_package(&task_package.id, &created_groups[0].id, &vec![NewTempTask {
            task_doc_id: task.id.clone(),
            task_type: task.task_type.clone(),
        }]).unwrap();

        let solution_attempt = util::create_solution_attempt(&test_app.group_repo, &created_user.id, &task_package.id, &created_groups[0].id);

        let resp = test_app
        .call(
            &get_path(&created_groups[0].id, &solution_attempt.solution_attempt.id),
            SCOPE,
            finish_solution_attempt,
            test_app.invalid_authorizate(TestRequest::post(), &created_user.id),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);
    }
}