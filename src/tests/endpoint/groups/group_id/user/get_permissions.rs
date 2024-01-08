//docu written with chat-gpt
#[cfg(test)]
mod permissions_tests {
    use actix_web::{test::{TestRequest, self}, http};
    use uuid::Uuid;

    use crate::{tests::{util, test::TestRepo}, models::{permissions::{NewRessource, OptionalUserAccessType, PermissionListResponse, PermissionListResponseWithCount}, util::AccessType, groups::NewUserPermission}};

    static SCOPE: &'static str = "/api/groups/{group_id}/user"; 
    
    fn get_path(group_id: &Uuid, query: &str) -> String {
        format!("/api/groups/{}/user/permissions{}", group_id, query)
    }

    /// # Test: `test_my_permissions_successfully`
    ///
    /// Validates the behavior of the `get_group_permissions` handler in Actix-Web when a user has permissions for the requested group. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a test user (`created_user`) and an example group (`created_groups`) associated with `created_user`.
    ///    - Creates resource permissions for the "Group" and "Permission" resources (`permissions`), allowing read access.
    ///    - Creates user permissions for `created_user`, granting read access to the "Group" and "Permission" resources.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `get_group_permissions` using the test application instance with a valid access token for `created_user`.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "OK" (200), indicating a successful request.
    ///    - Ensures that the response body contains the expected permissions for `created_user` and `created_groups`.
    ///    - Ensures that the test environment is cleaned up by clearing the database.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_my_permissions_successfully() {
        use crate::handlers::groups::group_id::user::user::get_group_permissions;

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
                &get_path(&created_groups[0].id, ""),
                SCOPE,
                get_group_permissions,
                test_app.valid_authorizate(TestRequest::get(), &created_user.id),
            )
            .await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let permission_response: PermissionListResponseWithCount = test::read_body_json(resp).await;
        util::assert_permission_response(&permission_response.permission_list, &user_permission_list);
        test_app.app_state.pgdb.clear_db();
    }

    /// # Test: `test_my_permissions_successfully_group_only`
    ///
    /// Validates the behavior of the `get_group_permissions` handler in Actix-Web when a user has permissions for the requested group only. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a test user (`created_user`) and an example group (`created_groups`) associated with `created_user`.
    ///    - Creates resource permissions for the "Group" and "Permission" resources (`permissions`), allowing read access.
    ///    - Creates user permissions for `created_user`, granting read access to the "Group" and "Permission" resources.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `get_group_permissions` using the test application instance with a valid access token for `created_user` and the `group_only` query parameter set to `true`.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "OK" (200), indicating a successful request.
    ///    - Ensures that the response body contains the expected permissions for `created_user` and the specified group (`created_groups`).
    ///    - Ensures that the test environment is cleaned up by clearing the database.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_my_permissions_successfully_group_only() {
        use crate::handlers::groups::group_id::user::user::get_group_permissions;

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
                &get_path(&created_groups[0].id, "?group_only=true"),
                SCOPE,
                get_group_permissions,
                test_app.valid_authorizate(TestRequest::get(), &created_user.id),
            )
            .await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let permission_response: PermissionListResponseWithCount = test::read_body_json(resp).await;
        util::assert_permission_response(&permission_response.permission_list, &vec![user_permission_list[0].clone()]);
        test_app.app_state.pgdb.clear_db();
    }

    /// # Test: `test_paging_my_permissions_successfully`
    ///
    /// Validates the behavior of the `get_group_permissions` handler in Actix-Web when paginating through user permissions. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a test user (`created_user`) and an example group (`created_groups`) associated with `created_user`.
    ///    - Creates resource permissions for the "Group," "Permission," and "Answer" resources (`permissions`), allowing read access.
    ///    - Creates user permissions for `created_user`, granting read access to the "Group," "Permission," and "Answer" resources.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `get_group_permissions` using the test application instance with a valid access token for `created_user` and pagination parameters (`page=1&limit=2`).
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "OK" (200), indicating a successful request.
    ///    - Ensures that the response body contains the expected permissions for `created_user` based on the specified pagination parameters.
    ///    - Ensures that the test environment is cleaned up by clearing the database.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_paging_my_permissions_successfully() {
        use crate::handlers::groups::group_id::user::user::get_group_permissions;

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
                &get_path(&created_groups[0].id, "?page=1&limit=2"),
                SCOPE,
                get_group_permissions,
                test_app.valid_authorizate(TestRequest::get(), &created_user.id),
            )
            .await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let permission_response: PermissionListResponseWithCount = test::read_body_json(resp).await;
        util::assert_permission_response(&permission_response.permission_list, &vec![user_permission_list[2].clone()]);
        test_app.app_state.pgdb.clear_db();
    }

    /// # Test: `test_paging_my_permissions_successfully_groups_only`
    ///
    /// Validates the behavior of the `get_group_permissions` handler in Actix-Web when paginating through user permissions with the `group_only` parameter. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a test user (`created_user`) and an example group (`created_groups`) associated with `created_user`.
    ///    - Creates resource permissions for the "Group," "Permission," "Answer," "A," and "B" resources (`permissions`), allowing read access.
    ///    - Creates user permissions for `created_user`, granting read access to the "Group," "Permission," "Answer," "A," and "B" resources.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `get_group_permissions` using the test application instance with a valid access token for `created_user` and pagination parameters (`page=1&limit=2&group_only=true`).
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "OK" (200), indicating a successful request.
    ///    - Ensures that the response body contains the expected permissions for `created_user` based on the specified pagination parameters and the `group_only` parameter.
    ///    - Ensures that the test environment is cleaned up by clearing the database.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_paging_my_permissions_successfully_groups_only() {
        use crate::handlers::groups::group_id::user::user::get_group_permissions;

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
            }, vec![AccessType::Read]), (NewRessource {
                key_name: &"Answer".to_string(),
                key_value: &"a".to_string(),
            }, vec![AccessType::Read]), (NewRessource {
                key_name: &"Answer".to_string(),
                key_value: &"b".to_string(),
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
            }],),
            (NewUserPermission {
                user_id: created_user.id,
                group_id: Some(created_groups[0].id),
                ressource: "a".to_string()
            },
            vec![OptionalUserAccessType {
                access_type: AccessType::Read,
                permission: Some(true),
                set_permission: Some(false),
                set_set_permission: None,
            }],),
            (NewUserPermission {
                user_id: created_user.id,
                group_id: Some(created_groups[0].id),
                ressource: "b".to_string()
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
                &get_path(&created_groups[0].id, "?page=1&limit=2&group_only=true"),
                SCOPE,
                get_group_permissions,
                test_app.valid_authorizate(TestRequest::get(), &created_user.id),
            )
            .await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let permission_response: PermissionListResponseWithCount = test::read_body_json(resp).await;
        util::assert_permission_response(&permission_response.permission_list, &vec![user_permission_list[4].clone()]);
        test_app.app_state.pgdb.clear_db();
    }

    /// # Test: `test_my_ressource_list_successfully`
    ///
    /// Validates the behavior of the `get_group_permissions` handler in Actix-Web when retrieving a list of permissions for a user associated with a specific group. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a test user (`created_user`) and an example group (`created_groups`) associated with `created_user`.
    ///    - Creates resource permissions for the "Group," "Permission," and "Answer" resources (`permissions`), allowing read access.
    ///    - Creates user permissions for `created_user`, granting read access to the "Group," "Permission," and "Answer" resources.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `get_group_permissions` using the test application instance with a valid access token for `created_user` and query parameters (`ressources[]=group,permission`).
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "OK" (200), indicating a successful request.
    ///    - Ensures that the response body contains the expected permissions for `created_user` based on the specified query parameters.
    ///    - Ensures that the test environment is cleaned up by clearing the database.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_my_ressource_list_successfully() {
        use crate::handlers::groups::group_id::user::user::get_group_permissions;

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
                &get_path(&created_groups[0].id, &format!("?ressources[]={},{}", user_permission_list[0].0.ressource, user_permission_list[1].0.ressource)),
                SCOPE,
                get_group_permissions,
                test_app.valid_authorizate(TestRequest::get(), &created_user.id),
            )
            .await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let permission_response: PermissionListResponseWithCount = test::read_body_json(resp).await;
        util::assert_permission_response(&permission_response.permission_list, &vec![user_permission_list[0].clone(), user_permission_list[1].clone()]);
        test_app.app_state.pgdb.clear_db();
    }

    /// # Test: `test_my_ressource_list_successfully_group_only`
    ///
    /// Validates the behavior of the `get_group_permissions` handler in Actix-Web when retrieving a list of permissions for a user associated with a specific group with the "group_only" flag. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a test user (`created_user`) and an example group (`created_groups`) associated with `created_user`.
    ///    - Creates resource permissions for the "Group," "Permission," and "Answer" resources (`permissions`), allowing read access.
    ///    - Creates user permissions for `created_user`, granting read access to the "Group," "Permission," and "Answer" resources.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `get_group_permissions` using the test application instance with a valid access token for `created_user` and query parameters (`ressources[]=group,answer&group_only=true`).
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "OK" (200), indicating a successful request.
    ///    - Ensures that the response body contains the expected permissions for `created_user` based on the specified query parameters and the "group_only" flag.
    ///    - Ensures that the test environment is cleaned up by clearing the database.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_my_ressource_list_successfully_group_only() {
        use crate::handlers::groups::group_id::user::user::get_group_permissions;

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
                &get_path(&created_groups[0].id, &format!("?ressources[]={},{}&group_only=true", user_permission_list[0].0.ressource, user_permission_list[2].0.ressource)),
                SCOPE,
                get_group_permissions,
                test_app.valid_authorizate(TestRequest::get(), &created_user.id),
            )
            .await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let permission_response: PermissionListResponseWithCount = test::read_body_json(resp).await;
        assert_eq!(permission_response.permission_list.len(), 1);
        util::assert_permission_response(&permission_response.permission_list, &vec![user_permission_list[2].clone()]);
        test_app.app_state.pgdb.clear_db();
    }

    /// # Test: `test_my_paging_ressource_list_successfully`
    ///
    /// Validates the behavior of the `get_group_permissions` handler in Actix-Web when retrieving a paginated list of permissions for a user associated with a specific group. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a test user (`created_user`) and an example group (`created_groups`) associated with `created_user`.
    ///    - Creates resource permissions for the "Group," "Permission," and "Answer" resources (`permissions`), allowing read access.
    ///    - Creates user permissions for `created_user`, granting read access to the "Group," "Permission," and "Answer" resources.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `get_group_permissions` using the test application instance with a valid access token for `created_user` and query parameters (`ressources[]=group,permission&page=1&limit=1`).
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "OK" (200), indicating a successful request.
    ///    - Ensures that the response body contains a paginated list of permissions for `created_user` based on the specified query parameters.
    ///    - Verifies that the list is limited to the specified limit (1) and starts from the specified page (1).
    ///    - Ensures that the test environment is cleaned up by clearing the database.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_my_paging_ressource_list_successfully() {
        use crate::handlers::groups::group_id::user::user::get_group_permissions;

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
                &get_path(&created_groups[0].id, &format!("?ressources[]={},{}&page=1&limit=1", user_permission_list[0].0.ressource, user_permission_list[1].0.ressource)),
                SCOPE,
                get_group_permissions,
                test_app.valid_authorizate(TestRequest::get(), &created_user.id),
            )
            .await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let permission_response: PermissionListResponseWithCount = test::read_body_json(resp).await;
        util::assert_permission_response(&permission_response.permission_list, &vec![user_permission_list[1].clone()]);
        test_app.app_state.pgdb.clear_db();
    }

    /// # Test: `test_my_paging_ressource_list_successfully_group_only`
    ///
    /// Validates the behavior of the `get_group_permissions` handler in Actix-Web when retrieving a paginated list of permissions for a user associated with a specific group, considering only group-level permissions. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a test user (`created_user`) and an example group (`created_groups`) associated with `created_user`.
    ///    - Creates resource permissions for the "Group," "Permission," "Answer," "A," and "B" resources (`permissions`), allowing read access.
    ///    - Creates user permissions for `created_user`, granting read access to the "Group," "Permission," "Answer," "A," and "B" resources, both at the user and group levels.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `get_group_permissions` using the test application instance with a valid access token for `created_user` and query parameters (`ressources[]=group,permission&page=1&limit=1&group_only=true`).
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "OK" (200), indicating a successful request.
    ///    - Ensures that the response body contains an empty list of permissions, as the query parameters specify group-level permissions only.
    ///    - Verifies that the test environment is cleaned up by clearing the database.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_my_paging_ressource_list_successfully_group_only() {
        use crate::handlers::groups::group_id::user::user::get_group_permissions;

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
            }, vec![AccessType::Read,]), (NewRessource {
                key_name: &"A".to_string(),
                key_value: &"a".to_string(),
            }, vec![AccessType::Read,]), (NewRessource {
                key_name: &"B".to_string(),
                key_value: &"b".to_string(),
            }, vec![AccessType::Read,])
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
            }],),
            (NewUserPermission {
                user_id: created_user.id,
                group_id: Some(created_groups[0].id),
                ressource: "a".to_string()
            },
            vec![OptionalUserAccessType {
                access_type: AccessType::Read,
                permission: Some(true),
                set_permission: Some(false),
                set_set_permission: None,
            }],),
            (NewUserPermission {
                user_id: created_user.id,
                group_id: Some(created_groups[0].id),
                ressource: "b".to_string()
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
                &get_path(&created_groups[0].id, &format!("?ressources[]={},{}&page=1&limit=1&group_only=true", user_permission_list[0].0.ressource, user_permission_list[1].0.ressource)),
                SCOPE,
                get_group_permissions,
                test_app.valid_authorizate(TestRequest::get(), &created_user.id),
            )
            .await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let permission_response: PermissionListResponseWithCount = test::read_body_json(resp).await;
        assert_eq!(permission_response.permission_list.len(), 0);
        test_app.app_state.pgdb.clear_db();
    }

    /// # Test: `test_permissions_invalid_access_token`
    ///
    /// Validates the behavior of the `get_group_permissions` handler in Actix-Web when an invalid access token is provided. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a test user (`created_user`) and an example group (`created_groups`) associated with `created_user`.
    ///    - Creates resource permissions for the "Group" and "Permission" resources (`permissions`), allowing read access.
    ///    - Creates user permissions for `created_user`, granting read access to the "Group" and "Permission" resources, both at the user level.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `get_group_permissions` using the test application instance with an invalid access token for `created_user`.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "UNAUTHORIZED" (401), indicating an unauthorized request due to the invalid access token.
    ///    - Verifies that the test environment is cleaned up by clearing the database.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_permissions_invalid_access_token() {
        use crate::handlers::groups::group_id::user::user::get_group_permissions;

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
                &get_path(&created_groups[0].id, ""),
                SCOPE,
                get_group_permissions,
                test_app.invalid_authorizate(TestRequest::get(), &created_user.id),
            )
            .await;
        assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);
        test_app.app_state.pgdb.clear_db();
    }

    /// # Test: `test_permissions_no_permission`
    ///
    /// Validates the behavior of the `get_group_permissions` handler in Actix-Web when a user lacks the required permissions for the requested resources. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a test user (`created_user`) and an example group (`created_groups`) associated with `created_user`.
    ///    - Creates resource permissions for the "Group" and "Permission" resources (`permissions`), allowing read access.
    ///    - Creates user permissions for `created_user`, granting read access only to the "Group" resource.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `get_group_permissions` using the test application instance with a valid access token for `created_user`.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "FORBIDDEN" (403), indicating that the user lacks the required permissions for the requested resources.
    ///    - Verifies that the test environment is cleaned up by clearing the database.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_permissions_no_permission() {
        use crate::handlers::groups::group_id::user::user::get_group_permissions;

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
                group_id: None,
                ressource: "group".to_string()
            },
            vec![OptionalUserAccessType {
                access_type: AccessType::Read,
                permission: Some(true),
                set_permission: None,
                set_set_permission: None,
            }],),
        ];

        util::create_permissions_for_user(
            &test_app.group_repo,
            &user_permission_list,
        );

        let resp = test_app
            .call(
                &get_path(&created_groups[0].id, ""),
                SCOPE,
                get_group_permissions,
                test_app.valid_authorizate(TestRequest::get(), &created_user.id),
            )
            .await;
        assert_eq!(resp.status(), http::StatusCode::FORBIDDEN);
        test_app.app_state.pgdb.clear_db();
    }
}