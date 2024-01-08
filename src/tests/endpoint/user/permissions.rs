//doku written with chat-gpt
#[cfg(test)]
mod permissions_tests {
    use actix_web::{test::{TestRequest, self}, http};

    use crate::{tests::{util, test::TestRepo}, models::{permissions::{NewRessource, OptionalUserAccessType, PermissionListResponse}, util::AccessType, groups::NewUserPermission}};

    static SCOPE: &'static str = "/api/user"; 
    
    fn get_path(query: &str) -> String {
        format!("/api/user/permissions{}", query)
    }

    /// # Test: `test_permissions_successfully`
    ///
    /// Validates the behavior of the `get_my_global_permissions` handler in Actix-Web when retrieving a user's global permissions. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a standard test user (`created_user`), associated groups, and permissions.
    ///
    /// 2. Permission Assignment:
    ///    - Uses `util::create_permissions_for_user` to assign specific global permissions to the user for both `group` and `permission`.
    ///
    /// 3. Test Execution:
    ///    - Makes an API call to `get_my_global_permissions` using the test application instance.
    ///
    /// 4. Assertions:
    ///    - Verifies that the response status is "OK" (200).
    ///    - Parses the response body and asserts that the returned permission list matches the expected global permissions assigned to the user.
    ///
    /// 5. Cleanup:
    ///    - Clears the database to leave no side effects.
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_permissions_successfully() {
        use crate::handlers::user::user::get_my_global_permissions;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_groups = util::create_example_groups(&test_app.group_repo, 1, created_user.id.clone());

        let permissions = util::create_ressource(
            &test_app.permission_repo,
            &vec![(NewRessource {
                key_name: &"Group".to_string(),
                key_value: &"group".to_string(),
            }, vec![AccessType::Read]),(NewRessource {
                key_name: &"Permission".to_string(),
                key_value: &"permission".to_string(),
            }, vec![AccessType::Read])],
        );

        let user_permission_list: Vec<(NewUserPermission, Vec<OptionalUserAccessType>)> = vec![(NewUserPermission {
                user_id: created_user.id,
                group_id: Some(created_groups[0].id),
                ressource: "group".to_string()
            },
            vec![OptionalUserAccessType {
                access_type: AccessType::Read,
                permission: Some(true),
                set_permission: None,
                set_set_permission: None,
            }],),
            (NewUserPermission {
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
                get_my_global_permissions,
                test_app.valid_authorizate(TestRequest::get(), &created_user.id),
            )
            .await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let permission_response: PermissionListResponse = test::read_body_json(resp).await;
        util::assert_permission_response(&permission_response.permission_list, &user_permission_list);
        test_app.app_state.pgdb.clear_db();
    }

    /// # Test: `test_paging_permissions_successfully`
    ///
    /// Validates the behavior of the `get_my_global_permissions` handler in Actix-Web when retrieving a user's global permissions with paging. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a standard test user (`created_user`), associated groups, and permissions.
    ///
    /// 2. Permission Assignment:
    ///    - Uses `util::create_permissions_for_user` to assign specific global permissions to the user for `group`, `permission`, and `answer`.
    ///
    /// 3. Test Execution:
    ///    - Makes an API call to `get_my_global_permissions` with paging parameters (`page=1&limit=2`) using the test application instance.
    ///
    /// 4. Assertions:
    ///    - Verifies that the response status is "OK" (200).
    ///    - Parses the response body and asserts that the returned permission list matches the expected global permissions assigned to the user for the specified page and limit.
    ///
    /// 5. Cleanup:
    ///    - Clears the database to leave no side effects.
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_paging_permissions_successfully() {
        use crate::handlers::user::user::get_my_global_permissions;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_groups = util::create_example_groups(&test_app.group_repo, 1, created_user.id.clone());

        let permissions = util::create_ressource(
            &test_app.permission_repo,
            &vec![(NewRessource {
                key_name: &"Group".to_string(),
                key_value: &"group".to_string(),
            }, vec![AccessType::Read]),(NewRessource {
                key_name: &"Permission".to_string(),
                key_value: &"permission".to_string(),
            }, vec![AccessType::Read]), (NewRessource {
                key_name: &"Answer".to_string(),
                key_value: &"answer".to_string(),
            }, vec![AccessType::Read])
            ],
        );

        let user_permission_list: Vec<(NewUserPermission, Vec<OptionalUserAccessType>)> = vec![(NewUserPermission {
                user_id: created_user.id,
                group_id: None,
                ressource: "group".to_string()
            },
            vec![OptionalUserAccessType {
                access_type: AccessType::Read,
                permission: Some(true),
                set_permission: None,
                set_set_permission: None,
            }],),
            (NewUserPermission {
                user_id: created_user.id,
                group_id: None,
                ressource: "permission".to_string()
            },
            vec![OptionalUserAccessType {
                access_type: AccessType::Read,
                permission: Some(true),
                set_permission: None,
                set_set_permission: None,
            }],),
            (NewUserPermission {
                user_id: created_user.id,
                group_id: Some(created_groups[0].id),
                ressource: "answer".to_string()
            },
            vec![OptionalUserAccessType {
                access_type: AccessType::Read,
                permission: Some(true),
                set_permission: Some(false),
                set_set_permission: None,
            }],)
        ];

        util::create_permissions_for_user(
            &test_app.group_repo,
            &user_permission_list,
        );

        let resp = test_app
            .call(
                &get_path("?page=1&limit=2"),
                SCOPE,
                get_my_global_permissions,
                test_app.valid_authorizate(TestRequest::get(), &created_user.id),
            )
            .await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let permission_response: PermissionListResponse = test::read_body_json(resp).await;
        util::assert_permission_response(&permission_response.permission_list, &vec![user_permission_list[2].clone()]);
        test_app.app_state.pgdb.clear_db();
    }

    /// # Test: `test_ressource_list_successfully`
    ///
    /// Validates the behavior of the `get_my_global_permissions` handler in Actix-Web when retrieving a user's global permissions for specific resources. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a standard test user (`created_user`), associated groups, and permissions.
    ///
    /// 2. Permission Assignment:
    ///    - Uses `util::create_permissions_for_user` to assign specific global permissions to the user for `group`, `permission`, and `answer`.
    ///
    /// 3. Test Execution:
    ///    - Makes an API call to `get_my_global_permissions` with specific resources (`group` and `permission`) using the test application instance.
    ///
    /// 4. Assertions:
    ///    - Verifies that the response status is "OK" (200).
    ///    - Parses the response body and asserts that the returned permission list matches the expected global permissions assigned to the user for the specified resources.
    ///
    /// 5. Cleanup:
    ///    - Clears the database to leave no side effects.
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_ressource_list_successfully() {
        use crate::handlers::user::user::get_my_global_permissions;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_groups = util::create_example_groups(&test_app.group_repo, 1, created_user.id.clone());

        let permissions = util::create_ressource(
            &test_app.permission_repo,
            &vec![(NewRessource {
                key_name: &"Group".to_string(),
                key_value: &"group".to_string(),
            }, vec![AccessType::Read]),(NewRessource {
                key_name: &"Permission".to_string(),
                key_value: &"permission".to_string(),
            }, vec![AccessType::Read]), (NewRessource {
                key_name: &"Answer".to_string(),
                key_value: &"answer".to_string(),
            }, vec![AccessType::Read])
            ],
        );

        let user_permission_list: Vec<(NewUserPermission, Vec<OptionalUserAccessType>)> = vec![(NewUserPermission {
                user_id: created_user.id,
                group_id: None,
                ressource: "group".to_string()
            },
            vec![OptionalUserAccessType {
                access_type: AccessType::Read,
                permission: Some(true),
                set_permission: None,
                set_set_permission: None,
            }],),
            (NewUserPermission {
                user_id: created_user.id,
                group_id: None,
                ressource: "permission".to_string()
            },
            vec![OptionalUserAccessType {
                access_type: AccessType::Read,
                permission: Some(true),
                set_permission: None,
                set_set_permission: None,
            }],),
            (NewUserPermission {
                user_id: created_user.id,
                group_id: Some(created_groups[0].id),
                ressource: "answer".to_string()
            },
            vec![OptionalUserAccessType {
                access_type: AccessType::Read,
                permission: Some(true),
                set_permission: Some(false),
                set_set_permission: None,
            }],)
        ];

        util::create_permissions_for_user(
            &test_app.group_repo,
            &user_permission_list,
        );

        let resp = test_app
            .call(
                &get_path(&format!("?ressources[]={},{}", user_permission_list[0].0.ressource, user_permission_list[1].0.ressource)),
                SCOPE,
                get_my_global_permissions,
                test_app.valid_authorizate(TestRequest::get(), &created_user.id),
            )
            .await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let permission_response: PermissionListResponse = test::read_body_json(resp).await;
        util::assert_permission_response(&permission_response.permission_list, &vec![user_permission_list[0].clone(), user_permission_list[1].clone()]);
        test_app.app_state.pgdb.clear_db();
    }

    /// # Test: `test_paging_ressource_list_successfully`
    ///
    /// Validates the behavior of the `get_my_global_permissions` handler in Actix-Web when retrieving a user's global permissions for specific resources with paging. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a standard test user (`created_user`), associated groups, and permissions.
    ///
    /// 2. Permission Assignment:
    ///    - Uses `util::create_permissions_for_user` to assign specific global permissions to the user for `group`, `permission`, and `answer`.
    ///
    /// 3. Test Execution:
    ///    - Makes an API call to `get_my_global_permissions` with specific resources (`group` and `permission`) and paging parameters using the test application instance.
    ///
    /// 4. Assertions:
    ///    - Verifies that the response status is "OK" (200).
    ///    - Parses the response body and asserts that the returned permission list matches the expected global permissions assigned to the user for the specified resources and paging parameters.
    ///
    /// 5. Cleanup:
    ///    - Clears the database to leave no side effects.
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_paging_ressource_list_successfully() {
        use crate::handlers::user::user::get_my_global_permissions;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_groups = util::create_example_groups(&test_app.group_repo, 1, created_user.id.clone());

        let permissions = util::create_ressource(
            &test_app.permission_repo,
            &vec![(NewRessource {
                key_name: &"Group".to_string(),
                key_value: &"group".to_string(),
            }, vec![AccessType::Read]),(NewRessource {
                key_name: &"Permission".to_string(),
                key_value: &"permission".to_string(),
            }, vec![AccessType::Read]), (NewRessource {
                key_name: &"Answer".to_string(),
                key_value: &"answer".to_string(),
            }, vec![AccessType::Read])
            ],
        );

        let user_permission_list: Vec<(NewUserPermission, Vec<OptionalUserAccessType>)> = vec![(NewUserPermission {
                user_id: created_user.id,
                group_id: None,
                ressource: "group".to_string()
            },
            vec![OptionalUserAccessType {
                access_type: AccessType::Read,
                permission: Some(true),
                set_permission: None,
                set_set_permission: None,
            }],),
            (NewUserPermission {
                user_id: created_user.id,
                group_id: None,
                ressource: "permission".to_string()
            },
            vec![OptionalUserAccessType {
                access_type: AccessType::Read,
                permission: Some(true),
                set_permission: None,
                set_set_permission: None,
            }],),
            (NewUserPermission {
                user_id: created_user.id,
                group_id: Some(created_groups[0].id),
                ressource: "answer".to_string()
            },
            vec![OptionalUserAccessType {
                access_type: AccessType::Read,
                permission: Some(true),
                set_permission: Some(false),
                set_set_permission: None,
            }],)
        ];

        util::create_permissions_for_user(
            &test_app.group_repo,
            &user_permission_list,
        );

        let resp = test_app
            .call(
                &get_path(&format!("?ressources[]={},{}&page=1&limit=1", user_permission_list[0].0.ressource, user_permission_list[1].0.ressource)),
                SCOPE,
                get_my_global_permissions,
                test_app.valid_authorizate(TestRequest::get(), &created_user.id),
            )
            .await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let permission_response: PermissionListResponse = test::read_body_json(resp).await;
        util::assert_permission_response(&permission_response.permission_list, &vec![user_permission_list[1].clone()]);
        test_app.app_state.pgdb.clear_db();
    }

    /// # Test: `test_permissions_invalid_access_token`
    ///
    /// Validates the behavior of the `get_my_global_permissions` handler in Actix-Web when attempting to retrieve a user's global permissions with an invalid access token. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a standard test user (`created_user`) and associated groups and permissions.
    ///
    /// 2. Permission Assignment:
    ///    - Uses `util::create_permissions_for_user` to assign specific global permissions to the user for `group` and `permission`.
    ///
    /// 3. Test Execution:
    ///    - Makes an API call to `get_my_global_permissions` with an invalid access token using the test application instance.
    ///
    /// 4. Assertions:
    ///    - Verifies that the response status is "UNAUTHORIZED" (401), indicating the failure to authenticate the request with an invalid access token.
    ///
    /// 5. Cleanup:
    ///    - Clears the database to leave no side effects.
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_permissions_invalid_access_token() {
        use crate::handlers::user::user::get_my_global_permissions;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let permissions = util::create_ressource(
            &test_app.permission_repo,
            &vec![(NewRessource {
                key_name: &"Group".to_string(),
                key_value: &"group".to_string(),
            }, vec![AccessType::Read]),(NewRessource {
                key_name: &"Permission".to_string(),
                key_value: &"permission".to_string(),
            }, vec![AccessType::Read])],
        );

        let user_permission_list: Vec<(NewUserPermission, Vec<OptionalUserAccessType>)> = vec![(NewUserPermission {
                user_id: created_user.id,
                group_id: None,
                ressource: "group".to_string()
            },
            vec![OptionalUserAccessType {
                access_type: AccessType::Read,
                permission: Some(true),
                set_permission: None,
                set_set_permission: None,
            }],),
            (NewUserPermission {
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
                get_my_global_permissions,
                test_app.invalid_authorizate(TestRequest::get(), &created_user.id),
            )
            .await;
        assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);
        test_app.app_state.pgdb.clear_db();
    }
}