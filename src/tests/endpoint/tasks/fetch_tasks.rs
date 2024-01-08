//doku written with chat-gpt
#[cfg(test)]
mod fetch_tasks_tests {
    use actix_web::{test::{TestRequest, self}, http};

    use crate::{tests::{util, test::TestRepo}, models::{permissions::{NewRessource, OptionalUserAccessType}, groups::NewUserPermission, util::AccessType, task::TaskPagingResponse}};

    static SCOPE: &'static str = "/api/tasks"; 
    
    fn get_path(query: &str) -> String {
        format!("/api/tasks/{}", query)
    }

    /// # Test: `test_fetch_tasks`
    ///
    /// Validates the behavior of the `fetch_tasks` handler in Actix-Web when attempting to retrieve tasks. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a standard test user (`created_user`) and a task (`created_task`).
    ///
    /// 2. Permission Assignment:
    ///    - Uses `util::create_permissions_for_user` to assign read permission for tasks to the user.
    ///
    /// 3. Test Execution:
    ///    - Makes an API call to `fetch_tasks` using the test application instance with a valid access token.
    ///
    /// 4. Assertions:
    ///    - Verifies that the response status is "OK" (200), indicating a successful request.
    ///    - Validates the response body, ensuring it contains the correct number of tasks and matches the details of the created task.
    ///
    /// 5. Cleanup:
    ///    - Clears the MongoDB database to leave no side effects.
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_fetch_tasks() {
        use crate::handlers::tasks::tasks::fetch_tasks;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_task = util::create_task_mc(&test_app.app_state.mongodb).await;

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
                &get_path(""),
                SCOPE,
                fetch_tasks,
                test_app.valid_authorizate(TestRequest::get(), &created_user.id),
            )
            .await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let response: TaskPagingResponse = test::read_body_json(resp).await;
        
        assert_eq!(response.total_count, 1);
        assert_eq!(response.tasks.len(), 1);
        util::assert_task(&response.tasks[0], &created_task);
        test_app.app_state.mongodb.clear_db().await;
    }

    /// # Test: `test_fetch_tasks_by_task_type`
    ///
    /// Validates the behavior of the `fetch_tasks` handler in Actix-Web when attempting to retrieve tasks filtered by task type. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a standard test user (`created_user`) and two tasks (`created_task` and `second_created_task`).
    ///
    /// 2. Permission Assignment:
    ///    - Uses `util::create_permissions_for_user` to assign read permission for tasks to the user.
    ///
    /// 3. Test Execution:
    ///    - Makes an API call to `fetch_tasks` using the test application instance with a valid access token, filtering by a specific task type (`task_ids` parameter).
    ///
    /// 4. Assertions:
    ///    - Verifies that the response status is "OK" (200), indicating a successful request.
    ///    - Validates the response body, ensuring it contains the correct number of tasks and matches the details of the specified task (`created_task`).
    ///
    /// 5. Cleanup:
    ///    - Clears the MongoDB database to leave no side effects.
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_fetch_tasks_by_task_type() {
        use crate::handlers::tasks::tasks::fetch_tasks;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_task = util::create_task_mc(&test_app.app_state.mongodb).await;
        let second_created_task = util::create_task_mc(&test_app.app_state.mongodb).await;

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
                &get_path(&format!("?task_ids[]={}", &created_task.id)),
                SCOPE,
                fetch_tasks,
                test_app.valid_authorizate(TestRequest::get(), &created_user.id),
            )
            .await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let response: TaskPagingResponse = test::read_body_json(resp).await;
        
        assert_eq!(response.total_count, 2);
        assert_eq!(response.tasks.len(), 1);
        util::assert_task(&response.tasks[0], &created_task);
        test_app.app_state.mongodb.clear_db().await;
    }

    /// # Test: `test_fetch_tasks_by_paging`
    ///
    /// Validates the behavior of the `fetch_tasks` handler in Actix-Web when fetching tasks with paging parameters. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a standard test user (`created_user`) and three tasks (`created_task`, `second_created_task`, and `third_created_task`).
    ///
    /// 2. Permission Assignment:
    ///    - Uses `util::create_permissions_for_user` to assign read permission for tasks to the user.
    ///
    /// 3. Test Execution:
    ///    - Makes an API call to `fetch_tasks` using the test application instance with a valid access token and paging parameters (`page=1` and `limit=2`).
    ///
    /// 4. Assertions:
    ///    - Verifies that the response status is "OK" (200), indicating a successful request.
    ///    - Validates the response body, ensuring it contains the correct total count of tasks and matches the details of the specified task (`third_created_task`).
    ///
    /// 5. Cleanup:
    ///    - Clears the MongoDB database to leave no side effects.
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_fetch_tasks_by_paging() {
        use crate::handlers::tasks::tasks::fetch_tasks;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_task = util::create_task_mc(&test_app.app_state.mongodb).await;
        let second_created_task = util::create_task_mc(&test_app.app_state.mongodb).await;
        let third_created_task = util::create_task_mc(&test_app.app_state.mongodb).await;

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
                &get_path(&format!("?page=1&limit=2")),
                SCOPE,
                fetch_tasks,
                test_app.valid_authorizate(TestRequest::get(), &created_user.id),
            )
            .await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let response: TaskPagingResponse = test::read_body_json(resp).await;
        
        assert_eq!(response.total_count, 3);
        assert_eq!(response.tasks.len(), 1);
        util::assert_task(&response.tasks[0], &third_created_task);
        test_app.app_state.mongodb.clear_db().await;
    }

    /// # Test: `test_fetch_tasks_no_permission`
    ///
    /// Validates the behavior of the `fetch_tasks` handler in Actix-Web when a user lacks the necessary permissions to read tasks. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a standard test user (`created_user`) and a task (`created_task`).
    ///
    /// 2. Test Execution:
    ///    - Makes an API call to `fetch_tasks` using the test application instance with a valid access token for the user who lacks read permissions.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "FORBIDDEN" (403), indicating that the user does not have the required permissions to perform the action.
    ///
    /// 4. Cleanup:
    ///    - Clears the MongoDB database to leave no side effects.
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_fetch_tasks_no_permission() {
        use crate::handlers::tasks::tasks::fetch_tasks;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_task = util::create_task_mc(&test_app.app_state.mongodb).await;

        let permissions = util::create_ressource(
            &test_app.permission_repo,
            &vec![(NewRessource {
                key_name: &"Task".to_string(),
                key_value: &"task".to_string(),
            }, vec![AccessType::Read])
            ],
        );

        let resp = test_app
            .call(
                &get_path(""),
                SCOPE,
                fetch_tasks,
                test_app.valid_authorizate(TestRequest::get(), &created_user.id),
            )
            .await;
        assert_eq!(resp.status(), http::StatusCode::FORBIDDEN);

        test_app.app_state.mongodb.clear_db().await;
    }
}