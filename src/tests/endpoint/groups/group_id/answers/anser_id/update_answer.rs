//docu written with chat-gpt
#[cfg(test)]
mod update_answer_tests {
    use actix_web::{test::{TestRequest, self}, http};
    use mongodb::bson::doc;
    use serde_json::Value;
    use uuid::Uuid;

    use crate::{tests::{util::{self, create_other_test_user}, test::TestRepo}, models::{permissions::{NewRessource, OptionalUserAccessType}, groups::{NewUserPermission, CreateGroupResponse, GroupInfoResponse, GroupMetaDataResponse}, util::{AccessType, PagingSchema, OrderDir, Visibility, AnswerState}, task::{TaskPagingResponse, NewTempTask}, roles::{NewRole, UpdateRolePermission, NewRolePermission, NewRoleAccessType, UpdateRoleAccesType}, task_package::{TaskPackagesResponse, CreatedTaskPackageResponse}, solution_attempts::{CreatedSolutionAttemptResponse, SolutionAttemptWithAnswerListResponse}, answer::{AnswerResponse, UpdatedAnswerResponse}}};

    static SCOPE: &'static str = "/api/groups/{group_id}/answers/{answer_id}"; 
    
    fn get_path(group_id: &Uuid, answer_id: &Uuid) -> String {
        format!("/api/groups/{}/answers/{}/", group_id, answer_id)
    }

    fn create_body() -> Value {
        serde_json::json!({
            "solution": {
                "solution": 2
            }
        })
    }

    fn create_wrong_body() -> Value {
        serde_json::json!({
            "solution": {
                "s": 2
            }
        })
    }

    /// # Test: `update_answer`
    ///
    /// Validates the behavior of the `update_answer` handler in Actix-Web when updating a user answer for a task. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a test user (`created_user`).
    ///    - Creates an example group (`created_groups`) associated with `created_user`.
    ///    - Grants `created_user` sufficient permissions for updating answers.
    ///    - Creates a task package, a multiple-choice task, and a solution attempt by `created_user`.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `update_answer` using the test application instance with valid authentication for `created_user`.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "OK" (200), indicating a successful update.
    ///    - Asserts the correctness of the updated answer response, checking for properties like the answer ID and solution.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn update_answer() {
        use crate::handlers::groups::group_id::answers::answer_id::answer_id::update_answer;
        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_groups = util::create_example_groups(&test_app.group_repo, 1, created_user.id);

        util::create_permissions(&test_app.permission_repo, &test_app.group_repo, "user_answer", &created_user.id, &AccessType::Write);

        let task_package = util::create_task_package(&test_app.group_repo, &vec![], &created_groups[0].id);

        let task = util::create_task_mc(&test_app.app_state.mongodb).await;

        let tasks = test_app.group_repo.add_tasks_to_package(&task_package.id, &created_groups[0].id, &vec![NewTempTask {
            task_doc_id: task.id.clone(),
            task_type: task.task_type.clone(),
        }]).unwrap();

        let solution_attempt = util::create_solution_attempt(&test_app.group_repo, &created_user.id, &task_package.id, &created_groups[0].id);

        util::create_schema(&test_app.app_state.mongodb).await;

        let resp = test_app
        .call(
            &get_path(&created_groups[0].id, &solution_attempt.solution_list[0].answer_id),
            SCOPE,
            update_answer,
            test_app.valid_authorizate(TestRequest::patch().set_json(create_body()), &created_user.id),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let response: UpdatedAnswerResponse = test::read_body_json(resp).await;


        assert_eq!(response.id, solution_attempt.solution_list[0].answer_id);
        assert_eq!(response.solution, doc!{"solution": 2})
        // TODO: assert other things
    }

    /// # Test: `update_answer_other`
    ///
    /// Validates the behavior of the `update_answer` handler in Actix-Web when attempting to update a user answer for a task by a user who does not own the answer. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates two test users (`me` and `created_user`).
    ///    - Creates an example group (`created_groups`) associated with `created_user`.
    ///    - Grants `me` sufficient permissions for updating answers.
    ///    - Creates a task package, a multiple-choice task, and a solution attempt by `created_user`.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `update_answer` using the test application instance with valid authentication for `me`.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "FORBIDDEN" (403), indicating that the user does not have permission to update the answer.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn update_answer_other() {
        use crate::handlers::groups::group_id::answers::answer_id::answer_id::update_answer;
        let test_app = TestRepo::new().await;

        let me = util::create_other_test_user(&test_app.user_repo);
        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_groups = util::create_example_groups(&test_app.group_repo, 1, created_user.id);

        util::create_other_permissions(&test_app.permission_repo, &test_app.group_repo, "user_answer", &me.id, &AccessType::Write);

        let task_package = util::create_task_package(&test_app.group_repo, &vec![], &created_groups[0].id);

        let task = util::create_task_mc(&test_app.app_state.mongodb).await;

        let tasks = test_app.group_repo.add_tasks_to_package(&task_package.id, &created_groups[0].id, &vec![NewTempTask {
            task_doc_id: task.id.clone(),
            task_type: task.task_type.clone(),
        }]).unwrap();

        let solution_attempt = util::create_solution_attempt(&test_app.group_repo, &created_user.id, &task_package.id, &created_groups[0].id);

        util::create_schema(&test_app.app_state.mongodb).await;

        let resp = test_app
        .call(
            &get_path(&created_groups[0].id, &solution_attempt.solution_list[0].answer_id),
            SCOPE,
            update_answer,
            test_app.valid_authorizate(TestRequest::patch().set_json(create_body()), &me.id),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::FORBIDDEN);
    }

    /// # Test: `update_answer_wrong_schema`
    ///
    /// Validates the behavior of the `update_answer` handler in Actix-Web when attempting to update a user answer for a task with an invalid schema provided in the request body. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a test user (`created_user`).
    ///    - Creates an example group (`created_groups`) associated with `created_user`.
    ///    - Grants `created_user` sufficient permissions for updating answers.
    ///    - Creates a task package, a multiple-choice task, and a solution attempt by `created_user`.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `update_answer` using the test application instance with valid authentication for `created_user` and an invalid schema in the request body.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "BAD_REQUEST" (400), indicating that the request body contains an invalid schema.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn update_answer_wrong_schema() {
        use crate::handlers::groups::group_id::answers::answer_id::answer_id::update_answer;
        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_groups = util::create_example_groups(&test_app.group_repo, 1, created_user.id);

        util::create_permissions(&test_app.permission_repo, &test_app.group_repo, "user_answer", &created_user.id, &AccessType::Write);

        let task_package = util::create_task_package(&test_app.group_repo, &vec![], &created_groups[0].id);

        let task = util::create_task_mc(&test_app.app_state.mongodb).await;

        let tasks = test_app.group_repo.add_tasks_to_package(&task_package.id, &created_groups[0].id, &vec![NewTempTask {
            task_doc_id: task.id.clone(),
            task_type: task.task_type.clone(),
        }]).unwrap();

        let solution_attempt = util::create_solution_attempt(&test_app.group_repo, &created_user.id, &task_package.id, &created_groups[0].id);

        util::create_schema(&test_app.app_state.mongodb).await;

        let resp = test_app
        .call(
            &get_path(&created_groups[0].id, &solution_attempt.solution_list[0].answer_id),
            SCOPE,
            update_answer,
            test_app.valid_authorizate(TestRequest::patch().set_json(create_wrong_body()), &created_user.id),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST);
    }

    /// # Test: `update_answer_no_auth`
    ///
    /// Validates the behavior of the `update_answer` handler in Actix-Web when attempting to update a user answer without proper authentication. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a test user (`created_user`).
    ///    - Creates an example group (`created_groups`) associated with `created_user`.
    ///    - Grants `created_user` sufficient permissions for updating answers.
    ///    - Creates a task package, a multiple-choice task, and a solution attempt by `created_user`.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `update_answer` using the test application instance with invalid authentication (no authentication) for `created_user`.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "UNAUTHORIZED" (401), indicating that the request lacks proper authentication.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn update_answer_no_auth() {
        use crate::handlers::groups::group_id::answers::answer_id::answer_id::update_answer;
        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_groups = util::create_example_groups(&test_app.group_repo, 1, created_user.id);

        util::create_permissions(&test_app.permission_repo, &test_app.group_repo, "user_answer", &created_user.id, &AccessType::Write);

        let task_package = util::create_task_package(&test_app.group_repo, &vec![], &created_groups[0].id);

        let task = util::create_task_mc(&test_app.app_state.mongodb).await;

        let tasks = test_app.group_repo.add_tasks_to_package(&task_package.id, &created_groups[0].id, &vec![NewTempTask {
            task_doc_id: task.id.clone(),
            task_type: task.task_type.clone(),
        }]).unwrap();

        let solution_attempt = util::create_solution_attempt(&test_app.group_repo, &created_user.id, &task_package.id, &created_groups[0].id);

        util::create_schema(&test_app.app_state.mongodb).await;

        let resp = test_app
        .call(
            &get_path(&created_groups[0].id, &solution_attempt.solution_list[0].answer_id),
            SCOPE,
            update_answer,
            test_app.invalid_authorizate(TestRequest::patch().set_json(create_body()), &created_user.id),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);
    }

    /// # Test: `update_answer_no_permission`
    ///
    /// Validates the behavior of the `update_answer` handler in Actix-Web when attempting to update a user answer without the necessary permissions. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a test user (`created_user`).
    ///    - Creates an example group (`created_groups`) associated with `created_user`.
    ///    - Does not grant `created_user` the required permissions for updating answers.
    ///    - Creates a task package, a multiple-choice task, and a solution attempt by `created_user`.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `update_answer` using the test application instance with valid authentication for `created_user`.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "FORBIDDEN" (403), indicating that the user lacks the necessary permissions to update answers.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn update_answer_no_permission() {
        use crate::handlers::groups::group_id::answers::answer_id::answer_id::update_answer;
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

        util::create_schema(&test_app.app_state.mongodb).await;

        let resp = test_app
        .call(
            &get_path(&created_groups[0].id, &solution_attempt.solution_list[0].answer_id),
            SCOPE,
            update_answer,
            test_app.valid_authorizate(TestRequest::patch().set_json(create_body()), &created_user.id),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::FORBIDDEN);
    }
}