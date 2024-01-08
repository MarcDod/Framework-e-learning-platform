//Doku written with chat-gpt
#[cfg(test)]
mod add_permissions_tests {
    use actix_web::{test::{self,TestRequest}, http};
    use uuid::Uuid;

    use crate::{tests::{test::TestRepo, util}, models::{auth::RegisterUserSchema, users::UserResponse, permissions::{NewRessource, OptionalUserAccessType}, util::{AccessType, PagingSchema, OrderDir}, groups::{NewUserPermission, AddPermissionSchema, PermissionSchema, AddPermissionResponse}}};

    static SCOPE: &'static str = "/api/users/{user_id}"; 
    
    fn get_path(user_id: &Uuid, query: &str) -> String {
        format!("/api/users/{}/permissions{}", user_id, query)
    }

    /// Test for adding global permissions to a user.
    ///
    /// This test validates the functionality of the `add_permissions_to_user` handler in Actix-Web, specifically focusing on adding global permissions to a user. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a standard test user (`created_user`), associated groups, and permissions.
    ///
    /// 2. Permission Assignment:
    ///    - Uses `util::create_permissions_for_user` to assign specific permissions to the user.
    ///
    /// 3. Request Preparation:
    ///    - Prepares a JSON request body containing new permissions for the user.
    ///
    /// 4. Test Execution:
    ///    - Makes an API call to `add_permissions_to_user` using the test application instance.
    ///
    /// 5. Assertions:
    ///    - Verifies that the response status is "CREATED" (201).
    ///    - Parses the response body and asserts that the updated permissions match the expected values.
    ///
    /// 6. Database Verification:
    ///    - Queries the database to ensure that the user's permissions are correctly updated.
    ///
    /// 7. Cleanup:
    ///    - Clears the database to leave no side effects.
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_add_permission_global() {
        use crate::handlers::users::user_id::user_id::add_permissions_to_user;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_groups = util::create_example_groups(&test_app.group_repo, 1, created_user.id.clone());

        let permissions = util::create_ressource(
            &test_app.permission_repo,
            &vec![(NewRessource {
                key_name: &"Group".to_string(),
                key_value: &"group".to_string(),
            }, vec![AccessType::Read, AccessType::Write]),(NewRessource {
                key_name: &"Permission".to_string(),
                key_value: &"permission".to_string(),
            }, vec![AccessType::Read, AccessType::Write]), (NewRessource {
                key_name: &"Answer".to_string(),
                key_value: &"answer".to_string(),
            }, vec![AccessType::Read])
            ],
        );

        let user_permission_list: Vec<(NewUserPermission, Vec<OptionalUserAccessType>)> = vec![
            (NewUserPermission {
                user_id: created_user.id,
                group_id: None,
                ressource: "permission".to_string()
            },
            vec![OptionalUserAccessType {
                access_type: AccessType::Write,
                permission: Some(false),
                set_permission: Some(true),
                set_set_permission: Some(false),
            }],),
        ];  

        util::create_permissions_for_user(
            &test_app.group_repo,
            &user_permission_list,
        );

        let request_body = serde_json::json!({
            "new_permissions": [
                {
                    "value": "permission",
                    "permission_addons": [
                        {
                            "access_type": "Write",
                            "permission": true,
                        },
                    ]
                }
            ]
        });

        let resp = test_app
            .call(
                &get_path(&created_user.id, ""),
                SCOPE,
                add_permissions_to_user,
                test_app.valid_authorizate(TestRequest::post().set_json(&request_body), &created_user.id),
            )
            .await;

        assert_eq!(resp.status(), http::StatusCode::CREATED);
        
        let response: AddPermissionResponse = test::read_body_json(resp).await;
        assert_eq!(response.updated_permissions, vec!["permission"]);


        let data = test_app.permission_repo.fetch_user_permissions(&created_user.id, &PagingSchema {
            limit: 200,
            page: 0,
            order: OrderDir::DESC,
        }, &None, false, &None).unwrap();

        assert_eq!(data.total_count, 1);
        assert_eq!(data.permission_list.len(), 1);
        assert_eq!(data.permission_list[0].key_value, "permission".to_string());
        assert_eq!(data.permission_list[0].access_types.len(), 1);
        assert_eq!(data.permission_list[0].access_types[0].access_type, AccessType::Write);
        assert_eq!(data.permission_list[0].access_types[0].permission, true);
        assert_eq!(data.permission_list[0].access_types[0].set_permission, true);
        assert_eq!(data.permission_list[0].access_types[0].set_set_permission, false );

        test_app.app_state.pgdb.clear_db();
    }

    #[actix_web::test]
    #[serial_test::serial]
    async fn test_add_set_permission_global() {
        use crate::handlers::users::user_id::user_id::add_permissions_to_user;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_groups = util::create_example_groups(&test_app.group_repo, 1, created_user.id.clone());

        let permissions = util::create_ressource(
            &test_app.permission_repo,
            &vec![(NewRessource {
                key_name: &"Group".to_string(),
                key_value: &"group".to_string(),
            }, vec![AccessType::Read, AccessType::Write]),(NewRessource {
                key_name: &"Permission".to_string(),
                key_value: &"permission".to_string(),
            }, vec![AccessType::Read, AccessType::Write]), (NewRessource {
                key_name: &"Answer".to_string(),
                key_value: &"answer".to_string(),
            }, vec![AccessType::Read])
            ],
        );

        let user_permission_list: Vec<(NewUserPermission, Vec<OptionalUserAccessType>)> = vec![
            (NewUserPermission {
                user_id: created_user.id,
                group_id: None,
                ressource: "permission".to_string()
            },
            vec![OptionalUserAccessType {
                access_type: AccessType::Write,
                permission: Some(false),
                set_permission: Some(false),
                set_set_permission: Some(true),
            }],),
        ];  

        util::create_permissions_for_user(
            &test_app.group_repo,
            &user_permission_list,
        );

        let request_body = serde_json::json!({
            "new_permissions": [
                {
                    "value": "permission",
                    "permission_addons": [
                        {
                            "access_type": "Write",
                            "set_permission": true,
                        },
                    ]
                }
            ]
        });

        let resp = test_app
            .call(
                &get_path(&created_user.id, ""),
                SCOPE,
                add_permissions_to_user,
                test_app.valid_authorizate(TestRequest::post().set_json(&request_body), &created_user.id),
            )
            .await;

        assert_eq!(resp.status(), http::StatusCode::CREATED);
        
        let response: AddPermissionResponse = test::read_body_json(resp).await;
        assert_eq!(response.updated_permissions, vec!["permission"]);


        let data = test_app.permission_repo.fetch_user_permissions(&created_user.id, &PagingSchema {
            limit: 200,
            page: 0,
            order: OrderDir::DESC,
        }, &None, false, &None).unwrap();

        assert_eq!(data.total_count, 1);
        assert_eq!(data.permission_list.len(), 1);
        assert_eq!(data.permission_list[0].key_value, "permission".to_string());
        assert_eq!(data.permission_list[0].access_types.len(), 1);
        assert_eq!(data.permission_list[0].access_types[0].access_type, AccessType::Write);
        assert_eq!(data.permission_list[0].access_types[0].permission, false);
        assert_eq!(data.permission_list[0].access_types[0].set_permission, true);
        assert_eq!(data.permission_list[0].access_types[0].set_set_permission, true);

        test_app.app_state.pgdb.clear_db();
    }

    /// # Test: `test_add_set_set_permission_global`
    ///
    /// Validates the behavior of the `add_permissions_to_user` handler in Actix-Web when adding global permissions with specific `set_set_permission` settings. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a standard test user (`created_user`), associated groups, and permissions.
    ///
    /// 2. Permission Assignment:
    ///    - Uses `util::create_permissions_for_user` to assign specific permissions to the user, including `set_set_permission`.
    ///
    /// 3. Request Preparation:
    ///    - Prepares a JSON request body containing new permissions for the user with `set_set_permission` set to false.
    ///
    /// 4. Test Execution:
    ///    - Makes an API call to `add_permissions_to_user` using the test application instance.
    ///
    /// 5. Assertions:
    ///    - Verifies that the response status is "CREATED" (201).
    ///    - Parses the response body and asserts that the updated permissions match the expected values.
    ///
    /// 6. Database Verification:
    ///    - Queries the database to ensure that the user's permissions are correctly updated.
    ///
    /// 7. Cleanup:
    ///    - Verifies that the total count of permissions for the user is 0.
    ///    - Clears the database to leave no side effects.
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_add_set_set_permission_global() {
        use crate::handlers::users::user_id::user_id::add_permissions_to_user;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_groups = util::create_example_groups(&test_app.group_repo, 1, created_user.id.clone());

        let permissions = util::create_ressource(
            &test_app.permission_repo,
            &vec![(NewRessource {
                key_name: &"Group".to_string(),
                key_value: &"group".to_string(),
            }, vec![AccessType::Read, AccessType::Write]),(NewRessource {
                key_name: &"Permission".to_string(),
                key_value: &"permission".to_string(),
            }, vec![AccessType::Read, AccessType::Write]), (NewRessource {
                key_name: &"Answer".to_string(),
                key_value: &"answer".to_string(),
            }, vec![AccessType::Read])
            ],
        );

        let user_permission_list: Vec<(NewUserPermission, Vec<OptionalUserAccessType>)> = vec![
            (NewUserPermission {
                user_id: created_user.id,
                group_id: None,
                ressource: "permission".to_string()
            },
            vec![OptionalUserAccessType {
                access_type: AccessType::Write,
                permission: Some(false),
                set_permission: Some(false),
                set_set_permission: Some(true),
            }],),
        ];  

        util::create_permissions_for_user(
            &test_app.group_repo,
            &user_permission_list,
        );

        let request_body = serde_json::json!({
            "new_permissions": [
                {
                    "value": "permission",
                    "permission_addons": [
                        {
                            "access_type": "Write",
                            "set_set_permission": false,
                        },
                    ]
                }
            ]
        });

        let resp = test_app
            .call(
                &get_path(&created_user.id, ""),
                SCOPE,
                add_permissions_to_user,
                test_app.valid_authorizate(TestRequest::post().set_json(&request_body), &created_user.id),
            )
            .await;

        assert_eq!(resp.status(), http::StatusCode::CREATED);
        
        let response: AddPermissionResponse = test::read_body_json(resp).await;
        assert_eq!(response.updated_permissions, vec!["permission"]);


        let data = test_app.permission_repo.fetch_user_permissions(&created_user.id, &PagingSchema {
            limit: 200,
            page: 0,
            order: OrderDir::DESC,
        }, &None, false, &None).unwrap();

        assert_eq!(data.total_count, 0);

        test_app.app_state.pgdb.clear_db();
    }

    /// # Test: `test_add_no_permission_global`
    ///
    /// Validates the behavior of the `add_permissions_to_user` handler in Actix-Web when attempting to add permissions with explicit `permission` settings. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a standard test user (`created_user`), associated groups, and permissions.
    ///
    /// 2. Permission Assignment:
    ///    - Uses `util::create_permissions_for_user` to assign specific permissions to the user, including `permission`, `set_permission`, and `set_set_permission`.
    ///
    /// 3. Request Preparation:
    ///    - Prepares a JSON request body containing new permissions for the user with explicit `permission` settings.
    ///
    /// 4. Test Execution:
    ///    - Makes an API call to `add_permissions_to_user` using the test application instance.
    ///
    /// 5. Assertions:
    ///    - Verifies that the response status is "CREATED" (201).
    ///    - Parses the response body and asserts that no permissions are updated.
    ///
    /// 6. Database Verification:
    ///    - Queries the database to ensure that the user's total permissions count is 2.
    ///    - Verifies individual permission details in the database.
    ///
    /// 7. Cleanup:
    ///    - Clears the database to leave no side effects.
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_add_no_permission_global() {
        use crate::handlers::users::user_id::user_id::add_permissions_to_user;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_groups = util::create_example_groups(&test_app.group_repo, 1, created_user.id.clone());

        let permissions = util::create_ressource(
            &test_app.permission_repo,
            &vec![(NewRessource {
                key_name: &"Group".to_string(),
                key_value: &"group".to_string(),
            }, vec![AccessType::Read, AccessType::Write]),(NewRessource {
                key_name: &"Permission".to_string(),
                key_value: &"permission".to_string(),
            }, vec![AccessType::Read, AccessType::Write]), (NewRessource {
                key_name: &"Answer".to_string(),
                key_value: &"answer".to_string(),
            }, vec![AccessType::Read])
            ],
        );

        let user_permission_list: Vec<(NewUserPermission, Vec<OptionalUserAccessType>)> = vec![
            (NewUserPermission {
                user_id: created_user.id,
                group_id: None,
                ressource: "permission".to_string()
            },
            vec![OptionalUserAccessType {
                access_type: AccessType::Write,
                permission: Some(true),
                set_permission: Some(false),
                set_set_permission: Some(false),
            }],),
            (NewUserPermission {
                user_id: created_user.id,
                group_id: None,
                ressource: "group".to_string()
            },
            vec![OptionalUserAccessType {
                access_type: AccessType::Read,
                permission: Some(false),
                set_permission: Some(true),
                set_set_permission: Some(false),
            }],),
        ];  

        util::create_permissions_for_user(
            &test_app.group_repo,
            &user_permission_list,
        );

        let request_body = serde_json::json!({
            "new_permissions": [
                {
                    "value": "permission",
                    "permission_addons": [
                        {
                            "access_type": "Write",
                            "permission": false,
                        },
                    ]
                },
                {
                    "value": "group",
                    "permission_addons": [
                        {
                            "access_type": "Read",
                            "set_permission": true,
                        },
                    ]
                }
            ]
        });

        let resp = test_app
            .call(
                &get_path(&created_user.id, ""),
                SCOPE,
                add_permissions_to_user,
                test_app.valid_authorizate(TestRequest::post().set_json(&request_body), &created_user.id),
            )
            .await;

        assert_eq!(resp.status(), http::StatusCode::CREATED);
        
        let response: AddPermissionResponse = test::read_body_json(resp).await;
        assert!(response.updated_permissions.is_empty());


        let data = test_app.permission_repo.fetch_user_permissions(&created_user.id, &PagingSchema {
            limit: 200,
            page: 0,
            order: OrderDir::DESC,
        }, &None, false, &None).unwrap();

        assert_eq!(data.total_count, 2);

        assert_eq!(data.permission_list.len(), 2);
        assert_eq!(data.permission_list[0].key_value, "permission".to_string());
        assert_eq!(data.permission_list[0].access_types.len(), 1);
        assert_eq!(data.permission_list[0].access_types[0].access_type, AccessType::Write);
        assert_eq!(data.permission_list[0].access_types[0].permission, true);
        assert_eq!(data.permission_list[0].access_types[0].set_permission, false);
        assert_eq!(data.permission_list[0].access_types[0].set_set_permission, false);
        assert_eq!(data.permission_list[1].key_value, "group".to_string());
        assert_eq!(data.permission_list[1].access_types.len(), 1);
        assert_eq!(data.permission_list[1].access_types[0].access_type, AccessType::Read);
        assert_eq!(data.permission_list[1].access_types[0].permission, false);
        assert_eq!(data.permission_list[1].access_types[0].set_permission, true);
        assert_eq!(data.permission_list[1].access_types[0].set_set_permission, false);

        test_app.app_state.pgdb.clear_db();
    }

    /// # Test: `test_add_ok_and_no_permission_global`
    ///
    /// Validates the behavior of the `add_permissions_to_user` handler in Actix-Web when adding permissions with both successful and unsuccessful scenarios. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a standard test user (`created_user`), associated groups, and permissions.
    ///
    /// 2. Permission Assignment:
    ///    - Uses `util::create_permissions_for_user` to assign specific permissions to the user, including `permission`, `set_permission`, and `set_set_permission`.
    ///
    /// 3. Request Preparation:
    ///    - Prepares a JSON request body containing new permissions for the user with explicit `permission` settings.
    ///
    /// 4. Test Execution:
    ///    - Makes an API call to `add_permissions_to_user` using the test application instance.
    ///
    /// 5. Assertions:
    ///    - Verifies that the response status is "CREATED" (201).
    ///    - Parses the response body and asserts that only the successful permission update is reflected in the response.
    ///
    /// 6. Database Verification:
    ///    - Queries the database to ensure that the user's total permissions count is 2.
    ///    - Verifies individual permission details in the database.
    ///
    /// 7. Cleanup:
    ///    - Clears the database to leave no side effects.
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_add_ok_and_no_permission_global() {
        use crate::handlers::users::user_id::user_id::add_permissions_to_user;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_groups = util::create_example_groups(&test_app.group_repo, 1, created_user.id.clone());

        let permissions = util::create_ressource(
            &test_app.permission_repo,
            &vec![(NewRessource {
                key_name: &"Group".to_string(),
                key_value: &"group".to_string(),
            }, vec![AccessType::Read, AccessType::Write]),(NewRessource {
                key_name: &"Permission".to_string(),
                key_value: &"permission".to_string(),
            }, vec![AccessType::Read, AccessType::Write]), (NewRessource {
                key_name: &"Answer".to_string(),
                key_value: &"answer".to_string(),
            }, vec![AccessType::Read])
            ],
        );

        let user_permission_list: Vec<(NewUserPermission, Vec<OptionalUserAccessType>)> = vec![
            (NewUserPermission {
                user_id: created_user.id,
                group_id: None,
                ressource: "permission".to_string()
            },
            vec![OptionalUserAccessType {
                access_type: AccessType::Write,
                permission: Some(true),
                set_permission: Some(true),
                set_set_permission: Some(false),
            }],),
            (NewUserPermission {
                user_id: created_user.id,
                group_id: None,
                ressource: "group".to_string()
            },
            vec![OptionalUserAccessType {
                access_type: AccessType::Read,
                permission: Some(false),
                set_permission: Some(true),
                set_set_permission: Some(false),
            }],),
        ];  

        util::create_permissions_for_user(
            &test_app.group_repo,
            &user_permission_list,
        );

        let request_body = serde_json::json!({
            "new_permissions": [
                {
                    "value": "permission",
                    "permission_addons": [
                        {
                            "access_type": "Write",
                            "permission": false,
                        },
                    ]
                },
                {
                    "value": "group",
                    "permission_addons": [
                        {
                            "access_type": "Read",
                            "set_permission": true,
                        },
                    ]
                }
            ]
        });

        let resp = test_app
            .call(
                &get_path(&created_user.id, ""),
                SCOPE,
                add_permissions_to_user,
                test_app.valid_authorizate(TestRequest::post().set_json(&request_body), &created_user.id),
            )
            .await;

        assert_eq!(resp.status(), http::StatusCode::CREATED);
        
        let response: AddPermissionResponse = test::read_body_json(resp).await;
        assert_eq!(response.updated_permissions, vec!["permission"]);


        let data = test_app.permission_repo.fetch_user_permissions(&created_user.id, &PagingSchema {
            limit: 200,
            page: 0,
            order: OrderDir::DESC,
        }, &None, false, &None).unwrap();

        assert_eq!(data.total_count, 2);

        assert_eq!(data.permission_list.len(), 2);
        assert_eq!(data.permission_list[0].key_value, "permission".to_string());
        assert_eq!(data.permission_list[0].access_types.len(), 1);
        assert_eq!(data.permission_list[0].access_types[0].access_type, AccessType::Write);
        assert_eq!(data.permission_list[0].access_types[0].permission, false);
        assert_eq!(data.permission_list[0].access_types[0].set_permission, true);
        assert_eq!(data.permission_list[0].access_types[0].set_set_permission, false);
        assert_eq!(data.permission_list[1].key_value, "group".to_string());
        assert_eq!(data.permission_list[1].access_types.len(), 1);
        assert_eq!(data.permission_list[1].access_types[0].access_type, AccessType::Read);
        assert_eq!(data.permission_list[1].access_types[0].permission, false);
        assert_eq!(data.permission_list[1].access_types[0].set_permission, true);
        assert_eq!(data.permission_list[1].access_types[0].set_set_permission, false);

        test_app.app_state.pgdb.clear_db();
    }

    /// # Test: `test_add_permission`
    ///
    /// Validates the behavior of the `add_permissions_to_user` handler in Actix-Web when adding permissions with group-specific settings. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a standard test user (`created_user`), associated groups, and permissions.
    ///
    /// 2. Permission Assignment:
    ///    - Uses `util::create_permissions_for_user` to assign specific permissions to the user, including `permission`, `set_permission`, and `set_set_permission`.
    ///
    /// 3. Request Preparation:
    ///    - Prepares a JSON request body containing new permissions for the user with explicit `permission` settings and a specific group ID.
    ///
    /// 4. Test Execution:
    ///    - Makes an API call to `add_permissions_to_user` using the test application instance.
    ///
    /// 5. Assertions:
    ///    - Verifies that the response status is "CREATED" (201).
    ///    - Parses the response body and asserts that the updated permissions match the expected values.
    ///
    /// 6. Database Verification:
    ///    - Queries the database to ensure that the user's total permissions count is 2.
    ///    - Verifies individual permission details in the database, including the group-specific permission.
    ///
    /// 7. Cleanup:
    ///    - Clears the database to leave no side effects.
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_add_permission() {
        use crate::handlers::users::user_id::user_id::add_permissions_to_user;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_groups = util::create_example_groups(&test_app.group_repo, 1, created_user.id.clone());

        let permissions = util::create_ressource(
            &test_app.permission_repo,
            &vec![(NewRessource {
                key_name: &"Group".to_string(),
                key_value: &"group".to_string(),
            }, vec![AccessType::Read, AccessType::Write]),(NewRessource {
                key_name: &"Permission".to_string(),
                key_value: &"permission".to_string(),
            }, vec![AccessType::Read, AccessType::Write]), (NewRessource {
                key_name: &"Answer".to_string(),
                key_value: &"answer".to_string(),
            }, vec![AccessType::Read])
            ],
        );

        let user_permission_list: Vec<(NewUserPermission, Vec<OptionalUserAccessType>)> = vec![
            (NewUserPermission {
                user_id: created_user.id,
                group_id: None,
                ressource: "permission".to_string()
            },
            vec![OptionalUserAccessType {
                access_type: AccessType::Write,
                permission: Some(false),
                set_permission: Some(true),
                set_set_permission: Some(false),
            }],),
        ];  

        util::create_permissions_for_user(
            &test_app.group_repo,
            &user_permission_list,
        );

        let request_body = serde_json::json!({
            "new_permissions": [
                {
                    "value": "permission",
                    "permission_addons": [
                        {
                            "access_type": "Write",
                            "permission": true,
                        },
                    ]
                }
            ]
        });

        let resp = test_app
            .call(
                &get_path(&created_user.id, &format!("?group_id={}", created_groups[0].id)),
                SCOPE,
                add_permissions_to_user,
                test_app.valid_authorizate(TestRequest::post().set_json(&request_body), &created_user.id),
            )
            .await;

        assert_eq!(resp.status(), http::StatusCode::CREATED);
        
        let response: AddPermissionResponse = test::read_body_json(resp).await;
        assert_eq!(response.updated_permissions, vec!["permission"]);


        let data = test_app.permission_repo.fetch_user_permissions(&created_user.id, &PagingSchema {
            limit: 200,
            page: 0,
            order: OrderDir::DESC,
        }, &Some(created_groups[0].id), false, &None).unwrap();

        assert_eq!(data.total_count, 2);
        assert_eq!(data.permission_list.len(), 2);
        assert_eq!(data.permission_list[0].key_value, "permission".to_string());
        assert_eq!(data.permission_list[0].access_types.len(), 1);
        assert_eq!(data.permission_list[0].access_types[0].access_type, AccessType::Write);
        assert_eq!(data.permission_list[0].access_types[0].permission, false);
        assert_eq!(data.permission_list[0].access_types[0].set_permission, true);
        assert_eq!(data.permission_list[0].access_types[0].set_set_permission, false );
        assert_eq!(data.permission_list[0].key_value, "permission".to_string());
        assert_eq!(data.permission_list[0].access_types.len(), 1);
        assert_eq!(data.permission_list[1].access_types[0].access_type, AccessType::Write);
        assert_eq!(data.permission_list[1].access_types[0].permission, true);
        assert_eq!(data.permission_list[1].access_types[0].set_permission, false);
        assert_eq!(data.permission_list[1].access_types[0].set_set_permission, false );
        assert!(data.permission_list[1].group_id.is_some());
        assert_eq!(data.permission_list[1].group_id.as_ref().unwrap(), &created_groups[0].id);

        test_app.app_state.pgdb.clear_db();
    }

    /// # Test: `test_add_set_permission`
    ///
    /// Validates the behavior of the `add_permissions_to_user` handler in Actix-Web when adding permissions with group-specific settings, specifically focusing on setting the permission. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a standard test user (`created_user`), associated groups, and permissions.
    ///
    /// 2. Permission Assignment:
    ///    - Uses `util::create_permissions_for_user` to assign specific permissions to the user, including `permission`, `set_permission`, and `set_set_permission`.
    ///
    /// 3. Request Preparation:
    ///    - Prepares a JSON request body containing new permissions for the user with explicit `set_permission` settings and a specific group ID.
    ///
    /// 4. Test Execution:
    ///    - Makes an API call to `add_permissions_to_user` using the test application instance.
    ///
    /// 5. Assertions:
    ///    - Verifies that the response status is "CREATED" (201).
    ///    - Parses the response body and asserts that the updated permissions match the expected values.
    ///
    /// 6. Database Verification:
    ///    - Queries the database to ensure that the user's total permissions count is 2.
    ///    - Verifies individual permission details in the database, including the group-specific permission with the `set_permission` flag set.
    ///
    /// 7. Cleanup:
    ///    - Clears the database to leave no side effects.
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_add_set_permission() {
        use crate::handlers::users::user_id::user_id::add_permissions_to_user;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_groups = util::create_example_groups(&test_app.group_repo, 1, created_user.id.clone());

        let permissions = util::create_ressource(
            &test_app.permission_repo,
            &vec![(NewRessource {
                key_name: &"Group".to_string(),
                key_value: &"group".to_string(),
            }, vec![AccessType::Read, AccessType::Write]),(NewRessource {
                key_name: &"Permission".to_string(),
                key_value: &"permission".to_string(),
            }, vec![AccessType::Read, AccessType::Write]), (NewRessource {
                key_name: &"Answer".to_string(),
                key_value: &"answer".to_string(),
            }, vec![AccessType::Read])
            ],
        );

        let user_permission_list: Vec<(NewUserPermission, Vec<OptionalUserAccessType>)> = vec![
            (NewUserPermission {
                user_id: created_user.id,
                group_id: None,
                ressource: "permission".to_string()
            },
            vec![OptionalUserAccessType {
                access_type: AccessType::Write,
                permission: Some(false),
                set_permission: Some(false),
                set_set_permission: Some(true),
            }],),
        ];  

        util::create_permissions_for_user(
            &test_app.group_repo,
            &user_permission_list,
        );

        let request_body = serde_json::json!({
            "new_permissions": [
                {
                    "value": "permission",
                    "permission_addons": [
                        {
                            "access_type": "Write",
                            "set_permission": true,
                        },
                    ]
                }
            ]
        });

        let resp = test_app
            .call(
                &get_path(&created_user.id, &format!("?group_id={}", created_groups[0].id)),
                SCOPE,
                add_permissions_to_user,
                test_app.valid_authorizate(TestRequest::post().set_json(&request_body), &created_user.id),
            )
            .await;

        assert_eq!(resp.status(), http::StatusCode::CREATED);
        
        let response: AddPermissionResponse = test::read_body_json(resp).await;
        assert_eq!(response.updated_permissions, vec!["permission"]);


        let data = test_app.permission_repo.fetch_user_permissions(&created_user.id, &PagingSchema {
            limit: 200,
            page: 0,
            order: OrderDir::DESC,
        }, &Some(created_groups[0].id), false, &None).unwrap();

        assert_eq!(data.total_count, 2);
        assert_eq!(data.permission_list.len(), 2);
        assert_eq!(data.permission_list[0].key_value, "permission".to_string());
        assert_eq!(data.permission_list[0].access_types.len(), 1);
        assert_eq!(data.permission_list[0].access_types[0].access_type, AccessType::Write);
        assert_eq!(data.permission_list[0].access_types[0].permission, false);
        assert_eq!(data.permission_list[0].access_types[0].set_permission, false);
        assert_eq!(data.permission_list[0].access_types[0].set_set_permission, true);
        assert_eq!(data.permission_list[0].key_value, "permission".to_string());
        assert_eq!(data.permission_list[0].access_types.len(), 1);
        assert_eq!(data.permission_list[1].access_types[0].access_type, AccessType::Write);
        assert_eq!(data.permission_list[1].access_types[0].permission, false);
        assert_eq!(data.permission_list[1].access_types[0].set_permission, true);
        assert_eq!(data.permission_list[1].access_types[0].set_set_permission, false);
        assert!(data.permission_list[1].group_id.is_some());
        assert_eq!(data.permission_list[1].group_id.as_ref().unwrap(), &created_groups[0].id);

        test_app.app_state.pgdb.clear_db();
    }

    /// # Test: `test_add_set_set_permission`
    ///
    /// Validates the behavior of the `add_permissions_to_user` handler in Actix-Web when adding permissions with group-specific settings, specifically focusing on setting and clearing the `set_set_permission` flag. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a standard test user (`created_user`), associated groups, and permissions.
    ///
    /// 2. Permission Assignment:
    ///    - Uses `util::create_permissions_for_user` to assign specific permissions to the user, including `permission`, `set_permission`, and `set_set_permission`.
    ///
    /// 3. Request Preparation:
    ///    - Prepares a JSON request body containing new permissions for the user with explicit `set_set_permission` settings and a specific group ID.
    ///
    /// 4. Test Execution:
    ///    - Makes an API call to `add_permissions_to_user` using the test application instance.
    ///
    /// 5. Assertions:
    ///    - Verifies that the response status is "CREATED" (201).
    ///    - Parses the response body and asserts that the updated permissions match the expected values.
    ///
    /// 6. Database Verification:
    ///    - Queries the database to ensure that the user's total permissions count is 2.
    ///    - Verifies individual permission details in the database, including the group-specific permission with the `set_set_permission` flag set.
    ///
    /// 7. Cleanup:
    ///    - Clears the database to leave no side effects.
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_add_set_set_permission() {
        use crate::handlers::users::user_id::user_id::add_permissions_to_user;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_groups = util::create_example_groups(&test_app.group_repo, 1, created_user.id.clone());

        let permissions = util::create_ressource(
            &test_app.permission_repo,
            &vec![(NewRessource {
                key_name: &"Group".to_string(),
                key_value: &"group".to_string(),
            }, vec![AccessType::Read, AccessType::Write]),(NewRessource {
                key_name: &"Permission".to_string(),
                key_value: &"permission".to_string(),
            }, vec![AccessType::Read, AccessType::Write]), (NewRessource {
                key_name: &"Answer".to_string(),
                key_value: &"answer".to_string(),
            }, vec![AccessType::Read])
            ],
        );

        let user_permission_list: Vec<(NewUserPermission, Vec<OptionalUserAccessType>)> = vec![
            (NewUserPermission {
                user_id: created_user.id,
                group_id: None,
                ressource: "permission".to_string()
            },
            vec![OptionalUserAccessType {
                access_type: AccessType::Write,
                permission: Some(false),
                set_permission: Some(false),
                set_set_permission: Some(true),
            }],),
        ];  

        util::create_permissions_for_user(
            &test_app.group_repo,
            &user_permission_list,
        );

        let request_body = serde_json::json!({
            "new_permissions": [
                {
                    "value": "permission",
                    "permission_addons": [
                        {
                            "access_type": "Write",
                            "set_set_permission": true,
                        },
                    ]
                }
            ]
        });

        let resp = test_app
            .call(
                &get_path(&created_user.id, &format!("?group_id={}", created_groups[0].id)),
                SCOPE,
                add_permissions_to_user,
                test_app.valid_authorizate(TestRequest::post().set_json(&request_body), &created_user.id),
            )
            .await;

        assert_eq!(resp.status(), http::StatusCode::CREATED);
        
        let response: AddPermissionResponse = test::read_body_json(resp).await;
        assert_eq!(response.updated_permissions, vec!["permission"]);


        let data = test_app.permission_repo.fetch_user_permissions(&created_user.id, &PagingSchema {
            limit: 200,
            page: 0,
            order: OrderDir::DESC,
        }, &Some(created_groups[0].id), false, &None).unwrap();

        assert_eq!(data.total_count, 2);
        assert_eq!(data.permission_list.len(), 2);
        assert_eq!(data.permission_list[0].key_value, "permission".to_string());
        assert_eq!(data.permission_list[0].access_types.len(), 1);
        assert_eq!(data.permission_list[0].access_types[0].access_type, AccessType::Write);
        assert_eq!(data.permission_list[0].access_types[0].permission, false);
        assert_eq!(data.permission_list[0].access_types[0].set_permission, false);
        assert_eq!(data.permission_list[0].access_types[0].set_set_permission, true);
        assert_eq!(data.permission_list[0].key_value, "permission".to_string());
        assert_eq!(data.permission_list[0].access_types.len(), 1);
        assert_eq!(data.permission_list[1].access_types[0].access_type, AccessType::Write);
        assert_eq!(data.permission_list[1].access_types[0].permission, false);
        assert_eq!(data.permission_list[1].access_types[0].set_permission, false);
        assert_eq!(data.permission_list[1].access_types[0].set_set_permission, true);
        assert!(data.permission_list[1].group_id.is_some());
        assert_eq!(data.permission_list[1].group_id.as_ref().unwrap(), &created_groups[0].id);

        test_app.app_state.pgdb.clear_db();
    }

    /// # Test: `test_add_no_permission`
    ///
    /// Validates the behavior of the `add_permissions_to_user` handler in Actix-Web when attempting to add permissions with conflicting settings (e.g., explicit permission value and `set_permission` flag). The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a standard test user (`created_user`), associated groups, and permissions.
    ///
    /// 2. Permission Assignment:
    ///    - Uses `util::create_permissions_for_user` to assign specific permissions to the user, including conflicting settings for `permission`, `set_permission`, and `set_set_permission`.
    ///
    /// 3. Request Preparation:
    ///    - Prepares a JSON request body containing new conflicting permissions for the user with explicit values and flags.
    ///
    /// 4. Test Execution:
    ///    - Makes an API call to `add_permissions_to_user` using the test application instance.
    ///
    /// 5. Assertions:
    ///    - Verifies that the response status is "CREATED" (201).
    ///    - Parses the response body and asserts that no permissions were updated (`updated_permissions` is empty).
    ///
    /// 6. Database Verification:
    ///    - Queries the database to ensure that the user's total permissions count is 2.
    ///    - Verifies individual permission details in the database, including the conflicting settings.
    ///
    /// 7. Cleanup:
    ///    - Clears the database to leave no side effects.
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_add_no_permission() {
        use crate::handlers::users::user_id::user_id::add_permissions_to_user;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_groups = util::create_example_groups(&test_app.group_repo, 1, created_user.id.clone());

        let permissions = util::create_ressource(
            &test_app.permission_repo,
            &vec![(NewRessource {
                key_name: &"Group".to_string(),
                key_value: &"group".to_string(),
            }, vec![AccessType::Read, AccessType::Write]),(NewRessource {
                key_name: &"Permission".to_string(),
                key_value: &"permission".to_string(),
            }, vec![AccessType::Read, AccessType::Write]), (NewRessource {
                key_name: &"Answer".to_string(),
                key_value: &"answer".to_string(),
            }, vec![AccessType::Read])
            ],
        );

        let user_permission_list: Vec<(NewUserPermission, Vec<OptionalUserAccessType>)> = vec![
            (NewUserPermission {
                user_id: created_user.id,
                group_id: None,
                ressource: "permission".to_string()
            },
            vec![OptionalUserAccessType {
                access_type: AccessType::Write,
                permission: Some(true),
                set_permission: Some(false),
                set_set_permission: Some(false),
            }],),
            (NewUserPermission {
                user_id: created_user.id,
                group_id: None,
                ressource: "group".to_string()
            },
            vec![OptionalUserAccessType {
                access_type: AccessType::Read,
                permission: Some(false),
                set_permission: Some(true),
                set_set_permission: Some(false),
            }],),
        ];  

        util::create_permissions_for_user(
            &test_app.group_repo,
            &user_permission_list,
        );

        let request_body = serde_json::json!({
            "new_permissions": [
                {
                    "value": "permission",
                    "permission_addons": [
                        {
                            "access_type": "Write",
                            "permission": false,
                        },
                    ]
                },
                {
                    "value": "group",
                    "permission_addons": [
                        {
                            "access_type": "Read",
                            "set_permission": true,
                        },
                    ]
                }
            ]
        });

        let resp = test_app
            .call(
                &get_path(&created_user.id, &format!("?group_id={}", created_groups[0].id)),
                SCOPE,
                add_permissions_to_user,
                test_app.valid_authorizate(TestRequest::post().set_json(&request_body), &created_user.id),
            )
            .await;

        assert_eq!(resp.status(), http::StatusCode::CREATED);
        
        let response: AddPermissionResponse = test::read_body_json(resp).await;
        assert!(response.updated_permissions.is_empty());


        let data = test_app.permission_repo.fetch_user_permissions(&created_user.id, &PagingSchema {
            limit: 200,
            page: 0,
            order: OrderDir::DESC,
        }, &Some(created_groups[0].id), false, &None).unwrap();

        assert_eq!(data.total_count, 2);

        assert_eq!(data.permission_list.len(), 2);
        assert_eq!(data.permission_list[0].key_value, "permission".to_string());
        assert_eq!(data.permission_list[0].access_types.len(), 1);
        assert_eq!(data.permission_list[0].access_types[0].access_type, AccessType::Write);
        assert_eq!(data.permission_list[0].access_types[0].permission, true);
        assert_eq!(data.permission_list[0].access_types[0].set_permission, false);
        assert_eq!(data.permission_list[0].access_types[0].set_set_permission, false);
        assert_eq!(data.permission_list[1].key_value, "group".to_string());
        assert_eq!(data.permission_list[1].access_types.len(), 1);
        assert_eq!(data.permission_list[1].access_types[0].access_type, AccessType::Read);
        assert_eq!(data.permission_list[1].access_types[0].permission, false);
        assert_eq!(data.permission_list[1].access_types[0].set_permission, true);
        assert_eq!(data.permission_list[1].access_types[0].set_set_permission, false);

        test_app.app_state.pgdb.clear_db();
    }

    /// # Test: `test_add_ok_and_no_permission`
    ///
    /// Validates the behavior of the `add_permissions_to_user` handler in Actix-Web when adding permissions with different settings for an existing permission and a new one. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a standard test user (`created_user`), associated groups, and permissions.
    ///
    /// 2. Permission Assignment:
    ///    - Uses `util::create_permissions_for_user` to assign specific permissions to the user, including conflicting settings for `permission`, `set_permission`, and `set_set_permission`.
    ///
    /// 3. Request Preparation:
    ///    - Prepares a JSON request body containing new conflicting permissions for the user with explicit values and flags.
    ///
    /// 4. Test Execution:
    ///    - Makes an API call to `add_permissions_to_user` using the test application instance.
    ///
    /// 5. Assertions:
    ///    - Verifies that the response status is "CREATED" (201).
    ///    - Parses the response body and asserts that the updated permissions include only the newly added permission (`updated_permissions` contains "permission").
    ///
    /// 6. Database Verification:
    ///    - Queries the database to ensure that the user's total permissions count is 3.
    ///    - Verifies individual permission details in the database, including the conflicting settings and the newly added permission.
    ///
    /// 7. Cleanup:
    ///    - Clears the database to leave no side effects.
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_add_ok_and_no_permission() {
        use crate::handlers::users::user_id::user_id::add_permissions_to_user;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_groups = util::create_example_groups(&test_app.group_repo, 1, created_user.id.clone());

        let permissions = util::create_ressource(
            &test_app.permission_repo,
            &vec![(NewRessource {
                key_name: &"Group".to_string(),
                key_value: &"group".to_string(),
            }, vec![AccessType::Read, AccessType::Write]),(NewRessource {
                key_name: &"Permission".to_string(),
                key_value: &"permission".to_string(),
            }, vec![AccessType::Read, AccessType::Write]), (NewRessource {
                key_name: &"Answer".to_string(),
                key_value: &"answer".to_string(),
            }, vec![AccessType::Read])
            ],
        );

        let user_permission_list: Vec<(NewUserPermission, Vec<OptionalUserAccessType>)> = vec![
            (NewUserPermission {
                user_id: created_user.id,
                group_id: None,
                ressource: "permission".to_string()
            },
            vec![OptionalUserAccessType {
                access_type: AccessType::Write,
                permission: Some(true),
                set_permission: Some(true),
                set_set_permission: Some(false),
            }],),
            (NewUserPermission {
                user_id: created_user.id,
                group_id: None,
                ressource: "group".to_string()
            },
            vec![OptionalUserAccessType {
                access_type: AccessType::Read,
                permission: Some(false),
                set_permission: Some(true),
                set_set_permission: Some(false),
            }],),
        ];  

        util::create_permissions_for_user(
            &test_app.group_repo,
            &user_permission_list,
        );

        let request_body = serde_json::json!({
            "new_permissions": [
                {
                    "value": "permission",
                    "permission_addons": [
                        {
                            "access_type": "Write",
                            "permission": true,
                        },
                    ]
                },
                {
                    "value": "group",
                    "permission_addons": [
                        {
                            "access_type": "Read",
                            "set_permission": true,
                        },
                    ]
                }
            ]
        });

        let resp = test_app
            .call(
                &get_path(&created_user.id, &format!("?group_id={}", created_groups[0].id)),
                SCOPE,
                add_permissions_to_user,
                test_app.valid_authorizate(TestRequest::post().set_json(&request_body), &created_user.id),
            )
            .await;

        assert_eq!(resp.status(), http::StatusCode::CREATED);
        
        let response: AddPermissionResponse = test::read_body_json(resp).await;
        assert_eq!(response.updated_permissions, vec!["permission"]);


        let data = test_app.permission_repo.fetch_user_permissions(&created_user.id, &PagingSchema {
            limit: 200,
            page: 0,
            order: OrderDir::DESC,
        }, &Some(created_groups[0].id), false, &None).unwrap();

        assert_eq!(data.total_count, 3);

        assert_eq!(data.permission_list.len(), 3);
        assert_eq!(data.permission_list[0].key_value, "permission".to_string());
        assert_eq!(data.permission_list[0].access_types.len(), 1);
        assert_eq!(data.permission_list[0].access_types[0].access_type, AccessType::Write);
        assert_eq!(data.permission_list[0].access_types[0].permission, true);
        assert_eq!(data.permission_list[0].access_types[0].set_permission, true);
        assert_eq!(data.permission_list[0].access_types[0].set_set_permission, false);
        assert_eq!(data.permission_list[1].key_value, "group".to_string());
        assert_eq!(data.permission_list[1].access_types.len(), 1);
        assert_eq!(data.permission_list[1].access_types[0].access_type, AccessType::Read);
        assert_eq!(data.permission_list[1].access_types[0].permission, false);
        assert_eq!(data.permission_list[1].access_types[0].set_permission, true);
        assert_eq!(data.permission_list[1].access_types[0].set_set_permission, false);
        assert_eq!(data.permission_list[2].key_value, "permission".to_string());
        assert_eq!(data.permission_list[2].access_types.len(), 1);
        assert_eq!(data.permission_list[2].access_types[0].access_type, AccessType::Write);
        assert_eq!(data.permission_list[2].access_types[0].permission, true);
        assert_eq!(data.permission_list[2].access_types[0].set_permission, false);
        assert_eq!(data.permission_list[2].access_types[0].set_set_permission, false);
        assert!(data.permission_list[2].group_id.is_some());
        assert_eq!(data.permission_list[2].group_id.as_ref().unwrap(), &created_groups[0].id);

        test_app.app_state.pgdb.clear_db();
    }
}