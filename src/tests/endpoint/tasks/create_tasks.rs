//docu written with chat-gpt
#[cfg(test)]
mod create_tasks_tests {
    use actix_web::{test::{TestRequest, self}, http};
    use mongodb::bson::{Document, doc};
    use serde_json::Value;

    use crate::{tests::{util, test::TestRepo}, models::{permissions::{NewRessource, OptionalUserAccessType}, groups::NewUserPermission, util::{AccessType, PagingSchema, OrderDir}, task::{TaskPagingResponse, TaskDoc, CreateTaskResponse}}, handlers::tasks::tasks::create_task};

    static SCOPE: &'static str = "/api/tasks"; 
    
    fn get_path(query: &str) -> String {
        format!("/api/tasks/{}", query)
    }

    fn create_body() -> Value {
        serde_json::json!({
            "task_type": "Multiple-Choice",
            "task": {
                "question": "h",
                "answers": ["f"],
            },
            "solution": {
                "solution": 0,
            }
        })
    }

    fn create_illegal_body() -> Value {
        serde_json::json!({
            "task_type": "Multiple-Choice",
            "task": {
                "questio": "h",
                "answers": ["f"],
            },
            "solution": {
                "solution": 0,
            }
        })
    }

    /// # Test: `test_create_tasks`
    ///
    /// Validates the behavior of the `create_task` handler in Actix-Web when a user has the necessary permissions to create tasks. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a standard test user (`created_user`).
    ///    - Grants the user create permissions for tasks.
    ///    - Creates a schema in the MongoDB database.
    ///    - Attempts to make an API call to `create_task` using the test application instance with a valid access token for the user.
    ///
    /// 2. Assertions:
    ///    - Verifies that the response status is "OK" (200), indicating that the task creation was successful.
    ///    - Fetches all tasks from the MongoDB database and compares the result with the created task from the API response to ensure consistency.
    ///
    /// 3. Cleanup:
    ///    - Clears the MongoDB database to leave no side effects.
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_create_tasks() {
        use crate::handlers::tasks::tasks::create_task;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let permissions = util::create_ressource(
            &test_app.permission_repo,
            &vec![(NewRessource {
                key_name: &"Task".to_string(),
                key_value: &"task".to_string(),
            }, vec![AccessType::Create])
            ],
        );

        let user_permission_list: Vec<(NewUserPermission, Vec<OptionalUserAccessType>)> = vec![(NewUserPermission {
                user_id: created_user.id,
                group_id: None,
                ressource: "task".to_string()
            },
            vec![OptionalUserAccessType {
                access_type: AccessType::Create,
                permission: Some(true),
                set_permission: None,
                set_set_permission: None,
            }],)
        ];

        util::create_permissions_for_user(
            &test_app.group_repo,
            &user_permission_list,
        );

        util::create_schema(&test_app.app_state.mongodb).await;

        let resp = test_app
            .call(
                &get_path(""),
                SCOPE,
                create_task,
                test_app.valid_authorizate(TestRequest::post().set_json(create_body()), &created_user.id),
            )
            .await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let response: CreateTaskResponse = test::read_body_json(resp).await;
        
        let task = test_app.app_state.mongodb.fetch_all_tasks(&PagingSchema {
            limit: 200,
            page: 0,
            order: OrderDir::DESC,
        }, None, true).await.unwrap();

        assert_eq!(task.len(), 1);
        assert_eq!(&response.task.task, &task[0].task);
        assert_eq!(&response.task.solution, &task[0].solution);
        assert_eq!(&response.task.id, &task[0].id);
    }

    /// # Test: `test_create_tasks_no_schema`
    ///
    /// Validates the behavior of the `create_task` handler in Actix-Web when a user attempts to create a task without a schema. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a standard test user (`created_user`).
    ///    - Grants the user create permissions for tasks.
    ///    - Attempts to make an API call to `create_task` using the test application instance with a valid access token for the user but without providing a schema in the request body.
    ///
    /// 2. Assertions:
    ///    - Verifies that the response status is "Bad Request" (400), indicating a client error due to the missing schema in the request.
    ///    - Fetches all tasks from the MongoDB database and ensures that no tasks were created.
    ///
    /// 3. Cleanup:
    ///    - Clears the MongoDB database to leave no side effects.
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_create_tasks_no_schema() {
        use crate::handlers::tasks::tasks::create_task;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let permissions = util::create_ressource(
            &test_app.permission_repo,
            &vec![(NewRessource {
                key_name: &"Task".to_string(),
                key_value: &"task".to_string(),
            }, vec![AccessType::Create])
            ],
        );

        let user_permission_list: Vec<(NewUserPermission, Vec<OptionalUserAccessType>)> = vec![(NewUserPermission {
                user_id: created_user.id,
                group_id: None,
                ressource: "task".to_string()
            },
            vec![OptionalUserAccessType {
                access_type: AccessType::Create,
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
                create_task,
                test_app.valid_authorizate(TestRequest::post().set_json(create_body()), &created_user.id),
            )
            .await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST);

        let task = test_app.app_state.mongodb.fetch_all_tasks(&PagingSchema {
            limit: 200,
            page: 0,
            order: OrderDir::DESC,
        }, None, true).await.unwrap();

        assert_eq!(task.len(), 0);
    }

    /// # Test: `test_create_tasks_wrong_schema`
    ///
    /// Validates the behavior of the `create_task` handler in Actix-Web when a user attempts to create a task with an invalid schema. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a standard test user (`created_user`).
    ///    - Grants the user create permissions for tasks.
    ///    - Attempts to make an API call to `create_task` using the test application instance with a valid access token for the user but providing an invalid schema in the request body.
    ///
    /// 2. Assertions:
    ///    - Verifies that the response status is "Bad Request" (400), indicating a client error due to the invalid schema in the request.
    ///    - Fetches all tasks from the MongoDB database and ensures that no tasks were created.
    ///
    /// 3. Cleanup:
    ///    - Clears the MongoDB database to leave no side effects.
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_create_tasks_wrong_schema() {
        use crate::handlers::tasks::tasks::create_task;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let permissions = util::create_ressource(
            &test_app.permission_repo,
            &vec![(NewRessource {
                key_name: &"Task".to_string(),
                key_value: &"task".to_string(),
            }, vec![AccessType::Create])
            ],
        );

        let user_permission_list: Vec<(NewUserPermission, Vec<OptionalUserAccessType>)> = vec![(NewUserPermission {
                user_id: created_user.id,
                group_id: None,
                ressource: "task".to_string()
            },
            vec![OptionalUserAccessType {
                access_type: AccessType::Create,
                permission: Some(true),
                set_permission: None,
                set_set_permission: None,
            }],)
        ];

        util::create_permissions_for_user(
            &test_app.group_repo,
            &user_permission_list,
        );

        util::create_schema(&test_app.app_state.mongodb).await;

        let resp = test_app
            .call(
                &get_path(""),
                SCOPE,
                create_task,
                test_app.valid_authorizate(TestRequest::post().set_json(create_illegal_body()), &created_user.id),
            )
            .await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST);

        let task = test_app.app_state.mongodb.fetch_all_tasks(&PagingSchema {
            limit: 200,
            page: 0,
            order: OrderDir::DESC,
        }, None, true).await.unwrap();

        assert_eq!(task.len(), 0);
    }
    
    /// # Test: `test_create_tasks_no_permission`
    ///
    /// Validates the behavior of the `create_task` handler in Actix-Web when a user attempts to create a task without the necessary permissions. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a standard test user (`created_user`).
    ///    - Attempts to make an API call to `create_task` using the test application instance with a valid access token for the user but without granting the user create permissions for tasks.
    ///
    /// 2. Assertions:
    ///    - Verifies that the response status is "Forbidden" (403), indicating that the user does not have the necessary permissions to create a task.
    ///    - Fetches all tasks from the MongoDB database and ensures that no tasks were created.
    ///
    /// 3. Cleanup:
    ///    - Clears the MongoDB database to leave no side effects.
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_create_tasks_no_permission() {
        use crate::handlers::tasks::tasks::create_task;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        util::create_schema(&test_app.app_state.mongodb).await;

        let resp = test_app
            .call(
                &get_path(""),
                SCOPE,
                create_task,
                test_app.valid_authorizate(TestRequest::post().set_json(create_body()), &created_user.id),
            )
            .await;
        assert_eq!(resp.status(), http::StatusCode::FORBIDDEN);

        let task = test_app.app_state.mongodb.fetch_all_tasks(&PagingSchema {
            limit: 200,
            page: 0,
            order: OrderDir::DESC,
        }, None, true).await.unwrap();

        assert_eq!(task.len(), 0);
    }
}