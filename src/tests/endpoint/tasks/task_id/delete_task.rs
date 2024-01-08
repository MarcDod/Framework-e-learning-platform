//docu written with chat-gpt
#[cfg(test)]
mod delete_tasks_tests {
    use actix_web::{test::{TestRequest, self}, http};
    use mongodb::bson::{Document, doc};
    use serde_json::Value;
    use uuid::Uuid;

    use crate::{tests::{util, test::TestRepo}, models::{permissions::{NewRessource, OptionalUserAccessType}, groups::NewUserPermission, util::{AccessType, PagingSchema, OrderDir, State}, task::{TaskPagingResponse, TaskDoc, CreateTaskResponse}}, handlers::tasks::tasks::create_task};

    static SCOPE: &'static str = "/api/tasks/{task_id}"; 
    
    fn get_path(task_id: &Uuid) -> String {
        format!("/api/tasks/{}/", task_id)
    }

    /// # Test: `test_delete_task`
    ///
    /// Validates the behavior of the `delete_task` handler in Actix-Web when a user with the necessary permissions attempts to delete a task. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a standard test user (`created_user`).
    ///    - Creates a task with multiple-choice schema (`task`).
    ///
    /// 2. Execution:
    ///    - Makes an API call to `delete_task` using the test application instance with a valid access token for the user and the required delete permissions for tasks.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "No Content" (204).
    ///    - Verifies that the task has been soft-deleted (state set to `State::Deleted`).
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_delete_task() {
        use crate::handlers::tasks::task_id::task_id::delete_task;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let permissions = util::create_ressource(
            &test_app.permission_repo,
            &vec![(NewRessource {
                key_name: &"Task".to_string(),
                key_value: &"task".to_string(),
            }, vec![AccessType::Delete])
            ],
        );

        let user_permission_list: Vec<(NewUserPermission, Vec<OptionalUserAccessType>)> = vec![(NewUserPermission {
                user_id: created_user.id,
                group_id: None,
                ressource: "task".to_string()
            },
            vec![OptionalUserAccessType {
                access_type: AccessType::Delete,
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
                delete_task,
                test_app.valid_authorizate(TestRequest::delete(), &created_user.id),
            )
            .await;
        assert_eq!(resp.status(), http::StatusCode::NO_CONTENT);

        let tasks = test_app.app_state.mongodb.fetch_all_tasks(&PagingSchema {
            limit: 200,
            page: 0,
            order: OrderDir::DESC,
        }, None, false).await.unwrap();

        assert_eq!(tasks.len(), 1);
        assert_eq!(&task.task, &tasks[0].task);
        assert_eq!(&task.solution, &tasks[0].solution);
        assert_eq!(&task.id, &tasks[0].id);
        assert_eq!(tasks[0].state, State::Deleted);
    }

    /// # Test: `test_delete_task_not_existing`
    ///
    /// Validates the behavior of the `delete_task` handler in Actix-Web when attempting to delete a non-existing task. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a standard test user (`created_user`).
    ///    - Configures the user with the required delete permissions for tasks.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `delete_task` using the test application instance with a valid access token for the user.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "No Content" (204).
    ///    - Verifies that no tasks exist in the database after the deletion attempt.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_delete_task_not_existing() {
        use crate::handlers::tasks::task_id::task_id::delete_task;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let permissions = util::create_ressource(
            &test_app.permission_repo,
            &vec![(NewRessource {
                key_name: &"Task".to_string(),
                key_value: &"task".to_string(),
            }, vec![AccessType::Delete])
            ],
        );

        let user_permission_list: Vec<(NewUserPermission, Vec<OptionalUserAccessType>)> = vec![(NewUserPermission {
                user_id: created_user.id,
                group_id: None,
                ressource: "task".to_string()
            },
            vec![OptionalUserAccessType {
                access_type: AccessType::Delete,
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
                delete_task,
                test_app.valid_authorizate(TestRequest::delete(), &created_user.id),
            )
            .await;
        assert_eq!(resp.status(), http::StatusCode::NO_CONTENT);

        let tasks = test_app.app_state.mongodb.fetch_all_tasks(&PagingSchema {
            limit: 200,
            page: 0,
            order: OrderDir::DESC,
        }, None, false).await.unwrap();

        assert_eq!(tasks.len(), 0);
    }

    /// # Test: `test_delete_task_no_permission`
    ///
    /// Validates the behavior of the `delete_task` handler in Actix-Web when attempting to delete a task without the necessary permissions. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a standard test user (`created_user`).
    ///    - Creates a task (`task`) in the database.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `delete_task` using the test application instance with a valid access token for the user, but without the required delete permissions.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "Forbidden" (403).
    ///    - Verifies that the task remains in the database and is still in the "Active" state.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_delete_task_no_permission() {
        use crate::handlers::tasks::task_id::task_id::delete_task;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let task = util::create_task_mc(&test_app.app_state.mongodb).await;

        let resp = test_app
            .call(
                &get_path(&task.id),
                SCOPE,
                delete_task,
                test_app.valid_authorizate(TestRequest::delete(), &created_user.id),
            )
            .await;
        assert_eq!(resp.status(), http::StatusCode::FORBIDDEN);

        let tasks = test_app.app_state.mongodb.fetch_all_tasks(&PagingSchema {
            limit: 200,
            page: 0,
            order: OrderDir::DESC,
        }, None, false).await.unwrap();

        assert_eq!(tasks.len(), 1);
        assert_eq!(&task.task, &tasks[0].task);
        assert_eq!(&task.solution, &tasks[0].solution);
        assert_eq!(&task.id, &tasks[0].id);
        assert_eq!(tasks[0].state, State::Active);
    }
}