//docu written with chat-gpt
#[cfg(test)]
mod fetch_solution_tests {
    use actix_web::{test::{TestRequest, self}, http};
    use mongodb::bson::{Document, doc};
    use serde_json::Value;
    use uuid::Uuid;

    use crate::{tests::{util, test::TestRepo}, models::{permissions::{NewRessource, OptionalUserAccessType}, groups::NewUserPermission, util::{AccessType, PagingSchema, OrderDir, State}, task::{TaskPagingResponse, TaskDoc, CreateTaskResponse, TaskResponse, TaskSolution}}, handlers::tasks::tasks::create_task};

    static SCOPE: &'static str = "/api/tasks/{task_id}"; 
    
    fn get_path(task_id: &Uuid) -> String {
        format!("/api/tasks/{}/solution", task_id)
    }

    /// # Test: `fetch_solution`
    ///
    /// Validates the behavior of the `fetch_task_solution` handler in Actix-Web when a user attempts to retrieve the solution for a task. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a standard test user (`created_user`).
    ///    - Grants read permissions for solutions to the user.
    ///    - Creates a task (`task`) in the test MongoDB.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `fetch_task_solution` using the test application instance with a valid access token for the user.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "OK" (200).
    ///    - Compares the retrieved solution in the response with the solution of the created task.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn fetch_solution() {
        use crate::handlers::tasks::task_id::task_id::fetch_task_solution;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let permissions = util::create_ressource(
            &test_app.permission_repo,
            &vec![(NewRessource {
                key_name: &"Solution".to_string(),
                key_value: &"solution".to_string(),
            }, vec![AccessType::Read])
            ],
        );

        let user_permission_list: Vec<(NewUserPermission, Vec<OptionalUserAccessType>)> = vec![(NewUserPermission {
                user_id: created_user.id,
                group_id: None,
                ressource: "solution".to_string()
            },
            vec![OptionalUserAccessType {
                access_type: AccessType::Read,
                permission: Some(true),
                set_permission: None,
                set_set_permission: None,
            }],)
        ];

        util::create_permissions_for_user(
            &test_app.group_repo,
            &user_permission_list,
        );

        let task = util::create_task_mc(&test_app.app_state.mongodb).await;

        let resp = test_app
            .call(
                &get_path(&task.id),
                SCOPE,
                fetch_task_solution,
                test_app.valid_authorizate(TestRequest::get(), &created_user.id),
            )
            .await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let response: TaskSolution = test::read_body_json(resp).await;

        assert_eq!(&response.solution, &task.solution);
    }

    /// # Test: `fetch_solution_not_found`
    ///
    /// Validates the behavior of the `fetch_task_solution` handler in Actix-Web when a user attempts to retrieve the solution for a non-existent task. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a standard test user (`created_user`).
    ///    - Grants read permissions for solutions to the user.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `fetch_task_solution` using the test application instance with a valid access token for the user and an invalid task ID.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "Not Found" (404).
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn fetch_solution_not_found() {
        use crate::handlers::tasks::task_id::task_id::fetch_task_solution;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let permissions = util::create_ressource(
            &test_app.permission_repo,
            &vec![(NewRessource {
                key_name: &"Solution".to_string(),
                key_value: &"solution".to_string(),
            }, vec![AccessType::Read])
            ],
        );

        let user_permission_list: Vec<(NewUserPermission, Vec<OptionalUserAccessType>)> = vec![(NewUserPermission {
                user_id: created_user.id,
                group_id: None,
                ressource: "solution".to_string()
            },
            vec![OptionalUserAccessType {
                access_type: AccessType::Read,
                permission: Some(true),
                set_permission: None,
                set_set_permission: None,
            }],)
        ];

        util::create_permissions_for_user(
            &test_app.group_repo,
            &user_permission_list,
        );

        let resp = test_app
            .call(
                &get_path(&Uuid::new_v4()),
                SCOPE,
                fetch_task_solution,
                test_app.valid_authorizate(TestRequest::get(), &created_user.id),
            )
            .await;
        assert_eq!(resp.status(), http::StatusCode::NOT_FOUND);
    }

    /// # Test: `fetch_task_no_permission`
    ///
    /// Validates the behavior of the `fetch_task_solution` handler in Actix-Web when a user attempts to retrieve the solution for a task without the necessary permissions. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a standard test user (`created_user`).
    ///    - Creates a task with multiple-choice schema (`task`).
    ///
    /// 2. Execution:
    ///    - Makes an API call to `fetch_task_solution` using the test application instance with a valid access token for the user but without the required read permissions for solutions.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "Forbidden" (403).
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn fetch_task_no_permission() {
        use crate::handlers::tasks::task_id::task_id::fetch_task_solution;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let task = util::create_task_mc(&test_app.app_state.mongodb).await;

        let resp = test_app
            .call(
                &get_path(&task.id),
                SCOPE,
                fetch_task_solution,
                test_app.valid_authorizate(TestRequest::get(), &created_user.id),
            )
            .await;
        assert_eq!(resp.status(), http::StatusCode::FORBIDDEN);
    }
}