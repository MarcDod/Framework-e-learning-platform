//docu written with chat-gpt
#[cfg(test)]
mod fetch_schemas_tests {
    use actix_web::{test::{TestRequest, self}, http};

    use crate::{tests::{util, test::TestRepo}, models::{permissions::{NewRessource, OptionalUserAccessType}, groups::NewUserPermission, util::AccessType, task::{TaskPagingResponse, SchemaPagingResponse}}};

    static SCOPE: &'static str = "/api/tasks"; 
    
    fn get_path(query: &str) -> String {
        format!("/api/tasks/schemas{}", query)
    }

    /// # Test: `test_fetch_tasks`
    ///
    /// Validates the behavior of the `fetch_schemas` handler in Actix-Web when a user has the necessary permissions to read schemas. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a standard test user (`created_user`).
    ///    - Creates a permission for the user to read schemas (`"schema"`).
    ///    - Generates two schemas (`schema` and `schema_2`).
    ///
    /// 2. Test Execution:
    ///    - Makes an API call to `fetch_schemas` using the test application instance with a valid access token for the user who has read permissions.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "OK" (200), indicating a successful request.
    ///    - Checks that the response body contains a total count of 2 and includes both schemas in the `schemas` array.
    ///
    /// 4. Cleanup:
    ///    - Clears the MongoDB database to leave no side effects.
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_fetch_tasks() {
        use crate::handlers::tasks::tasks::fetch_schemas;
     
        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let permissions = util::create_ressource(
            &test_app.permission_repo,
            &vec![(NewRessource {
                key_name: &"Schema".to_string(),
                key_value: &"schema".to_string(),
            }, vec![AccessType::Read])
            ],
        );

        let user_permission_list: Vec<(NewUserPermission, Vec<OptionalUserAccessType>)> = vec![(NewUserPermission {
                user_id: created_user.id,
                group_id: None,
                ressource: "schema".to_string()
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

        let schema = util::create_schema(&test_app.app_state.mongodb).await;
        let schema_2 = util::create_schema(&test_app.app_state.mongodb).await;

        let resp = test_app
        .call(
            &get_path(""),
            SCOPE,
            fetch_schemas,
            test_app.valid_authorizate(TestRequest::get(), &created_user.id),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let response: SchemaPagingResponse = test::read_body_json(resp).await;
        assert_eq!(response.total_count, 2);
        assert_eq!(response.schemas.len(), 2);
        util::assert_schma(&response.schemas[0], &schema);
        util::assert_schma(&response.schemas[1], &schema_2);
    }

    /// # Test: `test_fetch_tasks_task_types`
    ///
    /// Validates the behavior of the `fetch_schemas` handler in Actix-Web when filtering schemas by task types. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a standard test user (`created_user`).
    ///    - Creates a permission for the user to read schemas (`"schema"`).
    ///    - Generates two schemas (`schema` and `schema_2`), where `schema` has the task type "Multiple-Choice."
    ///
    /// 2. Test Execution:
    ///    - Makes an API call to `fetch_schemas` using the test application instance with a valid access token for the user who has read permissions. The request includes a filter for task types (e.g., `"Multiple-Choice"`).
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "OK" (200), indicating a successful request.
    ///    - Checks that the response body contains a total count of 2 and includes only the schema that matches the specified task type in the `schemas` array.
    ///
    /// 4. Cleanup:
    ///    - Clears the MongoDB database to leave no side effects.
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_fetch_tasks_task_types() {
        use crate::handlers::tasks::tasks::fetch_schemas;
     
        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let permissions = util::create_ressource(
            &test_app.permission_repo,
            &vec![(NewRessource {
                key_name: &"Schema".to_string(),
                key_value: &"schema".to_string(),
            }, vec![AccessType::Read])
            ],
        );

        let user_permission_list: Vec<(NewUserPermission, Vec<OptionalUserAccessType>)> = vec![(NewUserPermission {
                user_id: created_user.id,
                group_id: None,
                ressource: "schema".to_string()
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

        let schema = util::create_schema(&test_app.app_state.mongodb).await;
        let schema_2 = util::create_schema_other(&test_app.app_state.mongodb).await;

        let resp = test_app
        .call(
            &get_path("?task_types[]=Multiple-Choice"),
            SCOPE,
            fetch_schemas,
            test_app.valid_authorizate(TestRequest::get(), &created_user.id),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let response: SchemaPagingResponse = test::read_body_json(resp).await;
        assert_eq!(response.total_count, 2);
        assert_eq!(response.schemas.len(), 1);
        util::assert_schma(&response.schemas[0], &schema);
    }

    /// # Test: `test_fetch_tasks_paging`
    ///
    /// Validates the behavior of the `fetch_schemas` handler in Actix-Web when using pagination. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a standard test user (`created_user`).
    ///    - Creates a permission for the user to read schemas (`"schema"`).
    ///    - Generates three schemas (`schema`, `schema_2`, and `schema_3`).
    ///
    /// 2. Test Execution:
    ///    - Makes an API call to `fetch_schemas` using the test application instance with a valid access token for the user who has read permissions. The request includes pagination parameters (`page=1` and `limit=2`).
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "OK" (200), indicating a successful request.
    ///    - Checks that the response body contains a total count of 3 and includes only the schema that appears on the second page based on the specified limit (1 schema in this case).
    ///
    /// 4. Cleanup:
    ///    - Clears the MongoDB database to leave no side effects.
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_fetch_tasks_paging() {
        use crate::handlers::tasks::tasks::fetch_schemas;
     
        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let permissions = util::create_ressource(
            &test_app.permission_repo,
            &vec![(NewRessource {
                key_name: &"Schema".to_string(),
                key_value: &"schema".to_string(),
            }, vec![AccessType::Read])
            ],
        );

        let user_permission_list: Vec<(NewUserPermission, Vec<OptionalUserAccessType>)> = vec![(NewUserPermission {
                user_id: created_user.id,
                group_id: None,
                ressource: "schema".to_string()
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

        let schema = util::create_schema(&test_app.app_state.mongodb).await;
        let schema_2 = util::create_schema_other(&test_app.app_state.mongodb).await;
        let schema_3 = util::create_schema(&test_app.app_state.mongodb).await;

        let resp = test_app
        .call(
            &get_path("?page=1&limit=2"),
            SCOPE,
            fetch_schemas,
            test_app.valid_authorizate(TestRequest::get(), &created_user.id),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let response: SchemaPagingResponse = test::read_body_json(resp).await;
        assert_eq!(response.total_count, 3);
        assert_eq!(response.schemas.len(), 1);
        util::assert_schma(&response.schemas[0], &schema_3);
    }

    /// # Test: `test_fetch_tasks_no_permission`
    ///
    /// Validates the behavior of the `fetch_schemas` handler in Actix-Web when a user does not have the necessary permissions to read schemas. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a standard test user (`created_user`).
    ///    - Attempts to make an API call to `fetch_schemas` using the test application instance with a valid access token for the user who lacks read permissions.
    ///
    /// 2. Assertions:
    ///    - Verifies that the response status is "FORBIDDEN" (403), indicating that the user does not have the required permissions to perform the operation.
    ///
    /// 3. Cleanup:
    ///    - Clears the MongoDB database to leave no side effects.
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_fetch_tasks_no_permission() {
        use crate::handlers::tasks::tasks::fetch_schemas;
     
        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let schema = util::create_schema(&test_app.app_state.mongodb).await;
        let schema_2 = util::create_schema_other(&test_app.app_state.mongodb).await;
        let schema_3 = util::create_schema(&test_app.app_state.mongodb).await;

        let resp = test_app
        .call(
            &get_path(""),
            SCOPE,
            fetch_schemas,
            test_app.valid_authorizate(TestRequest::get(), &created_user.id),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::FORBIDDEN);
    }
}