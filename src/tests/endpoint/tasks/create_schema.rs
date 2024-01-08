//docu written with chat-gpt
#[cfg(test)]
mod create_schema_tests {
    use actix_web::{test::{TestRequest, self}, http};
    use serde_json::Value;

    use crate::{tests::{util, test::TestRepo}, models::{permissions::{NewRessource, OptionalUserAccessType}, groups::NewUserPermission, util::{AccessType, PagingSchema, OrderDir}, task::TaskPagingResponse}};

    static SCOPE: &'static str = "/api/tasks"; 
    
    fn get_path() -> String {
        format!("/api/tasks/schemas")
    }

    fn create_body() -> Value {
        serde_json::json!({
            "task_schema": {
                "type": "object",
                "properties": {
                "question": {
                    "type": "string"
                },
                "answers": {
                    "type": "array",
                    "items": {
                    "type": "string"
                    }
                }
                },
                "required": [
                "question",
                "answers"
                ],
                "additionalProperties": false
            },
            "task_type": "Multiple-Choice",
            "solution_schema": {
                "type": "object",
                "properties": {
                "solution": {
                    "type": "integer",
                    "minimum": 0
                }
                },
                "required": [
                "solution"
                ],
                "additionalProperties": false
            }
        })
    }

    fn create_illegal_body() -> Value {
        serde_json::json!({
            "task_schema": {
                "hello": "world"
            },
            "task_type": "Multiple-Choice",
            "solution_schema": {
                "type": "object",
                "properties": {
                "solution": {
                    "type": "integer",
                    "minimum": 0
                }
                },
                "required": [
                "solution"
                ],
                "additionalProperties": false
            }
        })
    }

    /// # Test: `test_create_schema`
    ///
    /// Validates the behavior of the `create_schema` handler in Actix-Web when a user with the necessary permissions attempts to create a new schema. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a standard test user (`created_user`).
    ///    - Grants the user create permissions for schemas.
    ///    - Attempts to make an API call to `create_schema` using the test application instance with a valid access token for the user.
    ///
    /// 2. Assertions:
    ///    - Verifies that the response status is "No Content" (204), indicating that the schema was created successfully.
    ///    - Fetches all schemas from the MongoDB database and ensures that the expected schema was created.
    ///
    /// 3. Cleanup:
    ///    - Clears the MongoDB database to leave no side effects.
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_create_schema() {
        use crate::handlers::tasks::tasks::create_schema;
     
        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let permissions = util::create_ressource(
            &test_app.permission_repo,
            &vec![(NewRessource {
                key_name: &"Schema".to_string(),
                key_value: &"schema".to_string(),
            }, vec![AccessType::Create])
            ],
        );

        let user_permission_list: Vec<(NewUserPermission, Vec<OptionalUserAccessType>)> = vec![(NewUserPermission {
                user_id: created_user.id,
                group_id: None,
                ressource: "schema".to_string()
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
            &get_path(),
            SCOPE,
            create_schema,
            test_app.valid_authorizate(TestRequest::post().set_json(create_body()), &created_user.id),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::NO_CONTENT);

        let schemas = test_app.app_state.mongodb.fetch_all_schemas(&PagingSchema {
            limit: 200,
            page: 0,
            order: OrderDir::DESC,
        }, None).await.unwrap();

        let example_schema = util::create_example_schema();

        assert_eq!(schemas.len(), 1);
        assert_eq!(schemas[0].task_type, "Multiple-Choice");
        assert_eq!(schemas[0].task_schema, example_schema.0);
        assert_eq!(schemas[0].solution_schema, example_schema.1);
    }

    /// # Test: `test_create_schema_no_permission`
    ///
    /// Validates the behavior of the `create_schema` handler in Actix-Web when a user without the necessary permissions attempts to create a new schema. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a standard test user (`created_user`).
    ///    - Attempts to make an API call to `create_schema` using the test application instance with a valid access token for the user, but without granting the user create permissions for schemas.
    ///
    /// 2. Assertions:
    ///    - Verifies that the response status is "Forbidden" (403), indicating that the user does not have the required permissions to create a schema.
    ///    - Fetches all schemas from the MongoDB database and ensures that no schema was created.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_create_schema_no_permission() {
        use crate::handlers::tasks::tasks::create_schema;
     
        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);


        let resp = test_app
        .call(
            &get_path(),
            SCOPE,
            create_schema,
            test_app.valid_authorizate(TestRequest::post().set_json(create_body()), &created_user.id),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::FORBIDDEN);

        let schemas = test_app.app_state.mongodb.fetch_all_schemas(&PagingSchema {
            limit: 200,
            page: 0,
            order: OrderDir::DESC,
        }, None).await.unwrap();

        assert_eq!(schemas.len(), 0);
    }

    /// # Test: `test_create_schema_already_existing`
    ///
    /// Validates the behavior of the `create_schema` handler in Actix-Web when a user attempts to create a schema with the same task type that already exists. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a standard test user (`created_user`).
    ///    - Grants the user create permissions for schemas.
    ///    - Creates an initial schema using the `util::create_schema` function.
    ///
    /// 2. Execution:
    ///    - Attempts to make an API call to `create_schema` using the test application instance with a valid access token for the user and a payload for creating a schema with the same task type.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "Conflict" (409), indicating that a schema with the same task type already exists.
    ///    - Fetches all schemas from the MongoDB database and ensures that the initial schema remains unchanged.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_create_schema_already_existing() {
        use crate::handlers::tasks::tasks::create_schema;
     
        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        util::create_schema(&test_app.app_state.mongodb).await;

        let permissions = util::create_ressource(
            &test_app.permission_repo,
            &vec![(NewRessource {
                key_name: &"Schema".to_string(),
                key_value: &"schema".to_string(),
            }, vec![AccessType::Create])
            ],
        );

        let user_permission_list: Vec<(NewUserPermission, Vec<OptionalUserAccessType>)> = vec![(NewUserPermission {
                user_id: created_user.id,
                group_id: None,
                ressource: "schema".to_string()
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
            &get_path(),
            SCOPE,
            create_schema,
            test_app.valid_authorizate(TestRequest::post().set_json(create_body()), &created_user.id),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::CONFLICT);

        let schemas = test_app.app_state.mongodb.fetch_all_schemas(&PagingSchema {
            limit: 200,
            page: 0,
            order: OrderDir::DESC,
        }, None).await.unwrap();

        assert_eq!(schemas.len(), 1);

        let example_schema = util::create_example_schema();

        assert_eq!(schemas.len(), 1);
        assert_eq!(schemas[0].task_type, "Multiple-Choice");
        assert_eq!(schemas[0].task_schema, example_schema.0);
        assert_eq!(schemas[0].solution_schema, example_schema.1);
    }

    /// # Test: `test_create_illegal_schema`
    ///
    /// Validates the behavior of the `create_schema` handler in Actix-Web when a user attempts to create a schema with an illegal payload. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a standard test user (`created_user`).
    ///    - Grants the user create permissions for schemas.
    ///
    /// 2. Execution:
    ///    - Attempts to make an API call to `create_schema` using the test application instance with a valid access token for the user and an illegal payload.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "Bad Request" (400), indicating that the payload is invalid.
    ///    - Fetches all schemas from the MongoDB database and ensures that no schemas have been created.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_create_illegal_schema() {
        use crate::handlers::tasks::tasks::create_schema;
     
        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let permissions = util::create_ressource(
            &test_app.permission_repo,
            &vec![(NewRessource {
                key_name: &"Schema".to_string(),
                key_value: &"schema".to_string(),
            }, vec![AccessType::Create])
            ],
        );

        let user_permission_list: Vec<(NewUserPermission, Vec<OptionalUserAccessType>)> = vec![(NewUserPermission {
                user_id: created_user.id,
                group_id: None,
                ressource: "schema".to_string()
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
            &get_path(),
            SCOPE,
            create_schema,
            test_app.valid_authorizate(TestRequest::post().set_json(create_illegal_body()), &created_user.id),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST);

        let schemas = test_app.app_state.mongodb.fetch_all_schemas(&PagingSchema {
            limit: 200,
            page: 0,
            order: OrderDir::DESC,
        }, None).await.unwrap();

        assert_eq!(schemas.len(), 0);
    }
}