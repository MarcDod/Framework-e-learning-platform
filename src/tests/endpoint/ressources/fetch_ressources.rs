//docu written with chat-gpt
#[cfg(test)]
mod fetch_ressources_tests {
    use actix_web::{test::{TestRequest, self}, http};

    use crate::{tests::{util, test::TestRepo}, models::{permissions::{NewRessource, OptionalUserAccessType, RessourceAndAccessTypesListWithCount}, groups::NewUserPermission, util::AccessType, task::TaskPagingResponse}};

    static SCOPE: &'static str = "/api/ressources"; 
    
    fn get_path(query: &str) -> String {
        format!("/api/ressources/{}", query)
    }

    /// # Test: `test_fetch_tasks`
    ///
    /// Validates the behavior of the `fetch_ressources` handler in Actix-Web when retrieving a list of resources and their associated access types. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a standard test user (`created_user`).
    ///    - Creates a task (`created_task`) in the database.
    ///    - Creates two resource permissions (`permissions`) in the database.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `fetch_ressources` using the test application instance with a valid access token for the user.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "OK" (200).
    ///    - Verifies that the response body contains the expected total count of resources.
    ///    - Verifies that the response body contains the expected list of resources, each with their associated access types, key name, and key value.
    ///
    /// 4. Cleanup:
    ///    - Clears the database after the test is completed.
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_fetch_tasks() {
        use crate::handlers::ressources::ressources::fetch_ressources;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_task = util::create_task_mc(&test_app.app_state.mongodb).await;

        let permissions = util::create_ressource(
            &test_app.permission_repo,
            &vec![(NewRessource {
                key_name: &"Permission".to_string(),
                key_value: &"permission".to_string(),
            }, vec![AccessType::Read]),
            (NewRessource {
                key_name: &"Group".to_string(),
                key_value: &"group".to_string(),
            }, vec![AccessType::Write])
            ],
        );

        let user_permission_list: Vec<(NewUserPermission, Vec<OptionalUserAccessType>)> = vec![(NewUserPermission {
                user_id: created_user.id,
                group_id: None,
                ressource: "permission".to_string()
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
                fetch_ressources,
                test_app.valid_authorizate(TestRequest::get(), &created_user.id),
            )
            .await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let response: RessourceAndAccessTypesListWithCount = test::read_body_json(resp).await;
        
        assert_eq!(response.total_count, 2);
        assert_eq!(response.ressources.len(), 2);
        assert_eq!(response.ressources[0].access_types, permissions[0].1);
        assert_eq!(response.ressources[0].key_value, permissions[0].0.key_value);
        assert_eq!(response.ressources[0].key_name, permissions[0].0.key_name);
        assert_eq!(response.ressources[1].access_types, permissions[1].1);
        assert_eq!(response.ressources[1].key_value, permissions[1].0.key_value);
        assert_eq!(response.ressources[1].key_name, permissions[1].0.key_name);

        test_app.app_state.mongodb.clear_db().await;
    }

    /// # Test: `test_fetch_ressource_by_ressource`
    ///
    /// Validates the behavior of the `fetch_ressources` handler in Actix-Web when retrieving a list of resources based on a specific resource key value. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a standard test user (`created_user`).
    ///    - Creates a task (`created_task`) in the database.
    ///    - Creates two resource permissions (`permissions`) in the database.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `fetch_ressources` using the test application instance with a valid access token for the user and a query parameter specifying a resource key value.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "OK" (200).
    ///    - Verifies that the response body contains the expected total count of resources.
    ///    - Verifies that the response body contains a list of resources with the specified resource key value, each with their associated access types, key name, and key value.
    ///
    /// 4. Cleanup:
    ///    - Clears the database after the test is completed.
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_fetch_ressource_by_ressource() {
        use crate::handlers::ressources::ressources::fetch_ressources;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_task = util::create_task_mc(&test_app.app_state.mongodb).await;

        let permissions = util::create_ressource(
            &test_app.permission_repo,
            &vec![(NewRessource {
                key_name: &"Permission".to_string(),
                key_value: &"permission".to_string(),
            }, vec![AccessType::Read]),
            (NewRessource {
                key_name: &"Group".to_string(),
                key_value: &"group".to_string(),
            }, vec![AccessType::Write])
            ],
        );

        let user_permission_list: Vec<(NewUserPermission, Vec<OptionalUserAccessType>)> = vec![(NewUserPermission {
                user_id: created_user.id,
                group_id: None,
                ressource: "permission".to_string()
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
                &get_path(&format!("?ressources[]={}", permissions[1].0.key_value)),
                SCOPE,
                fetch_ressources,
                test_app.valid_authorizate(TestRequest::get(), &created_user.id),
            )
            .await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let response: RessourceAndAccessTypesListWithCount = test::read_body_json(resp).await;
        
        assert_eq!(response.total_count, 2);
        assert_eq!(response.ressources.len(), 1);
        assert_eq!(response.ressources[0].access_types, permissions[1].1);
        assert_eq!(response.ressources[0].key_value, permissions[1].0.key_value);
        assert_eq!(response.ressources[0].key_name, permissions[1].0.key_name);

        test_app.app_state.mongodb.clear_db().await;
    }

    /// # Test: `test_fetch_ressource_by_paging`
    ///
    /// Validates the behavior of the `fetch_ressources` handler in Actix-Web when retrieving a paginated list of resources. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a standard test user (`created_user`).
    ///    - Creates a task (`created_task`) in the database.
    ///    - Creates three resource permissions (`permissions`) in the database.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `fetch_ressources` using the test application instance with a valid access token for the user and query parameters specifying pagination (`page=1` and `limit=2`).
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "OK" (200).
    ///    - Verifies that the response body contains the expected total count of resources.
    ///    - Verifies that the response body contains a paginated list of resources, limited to the specified page and limit, each with their associated access types, key name, and key value.
    ///
    /// 4. Cleanup:
    ///    - Clears the database after the test is completed.
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_fetch_ressource_by_paging() {
        use crate::handlers::ressources::ressources::fetch_ressources;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_task = util::create_task_mc(&test_app.app_state.mongodb).await;

        let permissions = util::create_ressource(
            &test_app.permission_repo,
            &vec![(NewRessource {
                key_name: &"Permission".to_string(),
                key_value: &"permission".to_string(),
            }, vec![AccessType::Read]),
            (NewRessource {
                key_name: &"Answer".to_string(),
                key_value: &"answer".to_string(),
            }, vec![AccessType::Read]),
            (NewRessource {
                key_name: &"Group".to_string(),
                key_value: &"group".to_string(),
            }, vec![AccessType::Write])
            ],
        );

        let user_permission_list: Vec<(NewUserPermission, Vec<OptionalUserAccessType>)> = vec![(NewUserPermission {
                user_id: created_user.id,
                group_id: None,
                ressource: "permission".to_string()
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
                fetch_ressources,
                test_app.valid_authorizate(TestRequest::get(), &created_user.id),
            )
            .await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let response: RessourceAndAccessTypesListWithCount = test::read_body_json(resp).await;
        
        assert_eq!(response.total_count, 3);
        assert_eq!(response.ressources.len(), 1);
        assert_eq!(response.ressources[0].access_types, permissions[2].1);
        assert_eq!(response.ressources[0].key_value, permissions[2].0.key_value);
        assert_eq!(response.ressources[0].key_name, permissions[2].0.key_name);

        test_app.app_state.mongodb.clear_db().await;
    }

    /// # Test: `test_fetch_ressource_no_permission`
    ///
    /// Validates the behavior of the `fetch_ressources` handler in Actix-Web when a user attempts to retrieve a list of resources without the necessary permissions. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a standard test user (`created_user`).
    ///    - Creates a task (`created_task`) in the database.
    ///    - Creates three resource permissions (`permissions`) in the database.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `fetch_ressources` using the test application instance with a valid access token for the user.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "Forbidden" (403), indicating that the user does not have the necessary permissions to access the resource list.
    ///
    /// 4. Cleanup:
    ///    - Clears the database after the test is completed.
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_fetch_ressource_no_permission() {
        use crate::handlers::ressources::ressources::fetch_ressources;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_task = util::create_task_mc(&test_app.app_state.mongodb).await;

        let permissions = util::create_ressource(
            &test_app.permission_repo,
            &vec![(NewRessource {
                key_name: &"Permission".to_string(),
                key_value: &"permission".to_string(),
            }, vec![AccessType::Read]),
            (NewRessource {
                key_name: &"Answer".to_string(),
                key_value: &"answer".to_string(),
            }, vec![AccessType::Read]),
            (NewRessource {
                key_name: &"Group".to_string(),
                key_value: &"group".to_string(),
            }, vec![AccessType::Write])
            ],
        );

        let resp = test_app
            .call(
                &get_path(&format!("")),
                SCOPE,
                fetch_ressources,
                test_app.valid_authorizate(TestRequest::get(), &created_user.id),
            )
            .await;
        assert_eq!(resp.status(), http::StatusCode::FORBIDDEN);

        test_app.app_state.mongodb.clear_db().await;
    }
}