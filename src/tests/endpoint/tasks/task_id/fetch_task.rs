//docu written with chat-gpt
#[cfg(test)]
mod fetch_tasks_tests {
    use actix_web::{test::{TestRequest, self}, http};
    use mongodb::bson::{Document, doc};
    use serde_json::Value;
    use uuid::Uuid;

    use crate::{tests::{util, test::TestRepo}, models::{permissions::{NewRessource, OptionalUserAccessType}, groups::NewUserPermission, util::{AccessType, PagingSchema, OrderDir, State}, task::{TaskPagingResponse, TaskDoc, CreateTaskResponse, TaskResponse}}, handlers::tasks::tasks::create_task};

    static SCOPE: &'static str = "/api/tasks/{task_id}"; 
    
    fn get_path(task_id: &Uuid) -> String {
        format!("/api/tasks/{}/", task_id)
    }

    /// # Test: `fetch_task`
    ///
    /// Validates the behavior of the `fetch_task` handler in Actix-Web when a user attempts to retrieve details for a specific task. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a standard test user (`created_user`).
    ///    - Grants the user read permissions for tasks.
    ///    - Creates a task (`task`) in the MongoDB database.
    ///
    /// 2. Execution:
    ///    - Attempts to make an API call to `fetch_task` using the test application instance with a valid access token for the user and the ID of the created task.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "OK" (200), indicating a successful request.
    ///    - Parses the response body into a `TaskResponse` and asserts that it matches the details of the created task.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn fetch_task() {
        use crate::handlers::tasks::task_id::task_id::fetch_task;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let permissions = util::create_ressource(
            &test_app.permission_repo,
            &vec![(NewRessource {
                key_name: &"Task".to_string(),
                key_value: &"task".to_string(),
            }, vec![AccessType::Read])
            ],
        );

        let user_permission_list: Vec<(NewUserPermission, Vec<OptionalUserAccessType>)> = vec![(NewUserPermission {
                user_id: created_user.id,
                group_id: None,
                ressource: "task".to_string()
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
                fetch_task,
                test_app.valid_authorizate(TestRequest::get(), &created_user.id),
            )
            .await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let response: TaskResponse = test::read_body_json(resp).await;

        util::assert_task(&response, &task);
    }

    /// # Test: `fetch_task_not_found`
    ///
    /// Validates the behavior of the `fetch_task` handler in Actix-Web when a user attempts to retrieve details for a non-existing task. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a standard test user (`created_user`).
    ///    - Grants the user read permissions for tasks.
    ///
    /// 2. Execution:
    ///    - Attempts to make an API call to `fetch_task` using the test application instance with a valid access token for the user and a non-existing task ID.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "Not Found" (404), indicating that the requested task was not found.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn fetch_task_not_found() {
        use crate::handlers::tasks::task_id::task_id::fetch_task;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let permissions = util::create_ressource(
            &test_app.permission_repo,
            &vec![(NewRessource {
                key_name: &"Task".to_string(),
                key_value: &"task".to_string(),
            }, vec![AccessType::Read])
            ],
        );

        let user_permission_list: Vec<(NewUserPermission, Vec<OptionalUserAccessType>)> = vec![(NewUserPermission {
                user_id: created_user.id,
                group_id: None,
                ressource: "task".to_string()
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
                fetch_task,
                test_app.valid_authorizate(TestRequest::get(), &created_user.id),
            )
            .await;
        assert_eq!(resp.status(), http::StatusCode::NOT_FOUND);
    }

    /// # Test: `fetch_task_no_permission`
    ///
    /// Validates the behavior of the `fetch_task` handler in Actix-Web when a user attempts to retrieve details for a task without the necessary permissions. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a standard test user (`created_user`).
    ///    - Creates a task (`task`) in the test MongoDB.
    ///
    /// 2. Execution:
    ///    - Attempts to make an API call to `fetch_task` using the test application instance with a valid access token for the user but without granting read permissions for tasks.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "Forbidden" (403), indicating that the user does not have the necessary permissions to retrieve task details.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn fetch_task_no_permission() {
        use crate::handlers::tasks::task_id::task_id::fetch_task;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let task = util::create_task_mc(&test_app.app_state.mongodb).await;

        let resp = test_app
            .call(
                &get_path(&task.id),
                SCOPE,
                fetch_task,
                test_app.valid_authorizate(TestRequest::get(), &created_user.id),
            )
            .await;
        assert_eq!(resp.status(), http::StatusCode::FORBIDDEN);
    }
}