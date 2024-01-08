//docu written with chat-gpt
#[cfg(test)]
mod permissions_tests {
    use actix_web::{test::{TestRequest, self}, http};
    use uuid::Uuid;

    use crate::{tests::{util, test::TestRepo}, models::{permissions::{NewRessource, OptionalUserAccessType, PermissionListResponse, PermissionListResponseWithCount}, util::AccessType, groups::NewUserPermission}};

    static SCOPE: &'static str = "/api/groups/{group_id}/users/{user_id}"; 
    
    fn get_path(group_id: &Uuid, user_id: &Uuid, query: &str) -> String {
        format!("/api/groups/{}/users/{}/permissions{}", group_id, user_id, query)
    }

    /// # Test: `test_my_permissions_successfully`
    ///
    /// Validates the behavior of the `get_group_permissions_from_user` handler in Actix-Web when a user successfully retrieves their permissions for a specific group. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a standard test user (`created_user`).
    ///    - Creates an example group (`created_groups`) associated with the user.
    ///    - Creates resource permissions for both the "Group" and "Permission" resources (`permissions`), allowing read access.
    ///    - Creates user permissions for the user, granting read access to the "Group" resource within the specified group and read access to the "Permission" resource globally.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `get_group_permissions_from_user` using the test application instance with a valid access token for the user. The user has read permission for both the group and global resources.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "OK" (200).
    ///    - Parses and asserts the response body, ensuring it contains the expected list of permissions for the user.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_my_permissions_successfully() {
        use crate::handlers::groups::group_id::users::user_id::user_id::get_group_permissions_from_user;

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
                &get_path(&created_groups[0].id, &created_user.id, ""),
                SCOPE,
                get_group_permissions_from_user,
                test_app.valid_authorizate(TestRequest::get(), &created_user.id),
            )
            .await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let permission_response: PermissionListResponseWithCount = test::read_body_json(resp).await;
        util::assert_permission_response(&permission_response.permission_list, &user_permission_list);
        test_app.app_state.pgdb.clear_db();
    }

    /// # Test: `test_other_permissions_no_permission`
    ///
    /// Validates the behavior of the `get_group_permissions_from_user` handler in Actix-Web when a user attempts to retrieve the permissions of another user for a specific group without the necessary permissions. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates two test users: `me` (an unauthorized user attempting to retrieve permissions) and `created_user` (the user whose permissions are being queried).
    ///    - Creates an example group (`created_groups`) associated with `created_user`.
    ///    - Creates resource permissions for both the "Group" and "Permission" resources (`permissions`), allowing read access.
    ///    - Creates user permissions for `created_user`, granting read access to the "Group" resource within the specified group, and for `me`, granting read access to the "Permission" resource globally.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_other_permissions_no_permission() {
        use crate::handlers::groups::group_id::users::user_id::user_id::get_group_permissions_from_user;

        let test_app = TestRepo::new().await;

        let me = util::create_other_test_user(&test_app.user_repo);

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
            }, vec![AccessType::Read, AccessType::Other])],
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
                user_id: me.id,
                group_id: None,
                ressource: "permission".to_string()
            },
            vec![OptionalUserAccessType {
                access_type: AccessType::Read,
                permission: Some(true),
                set_permission: None,
                set_set_permission: None,
            }])
        ];

        util::create_permissions_for_user(
            &test_app.group_repo,
            &user_permission_list,
        );

        let resp = test_app
            .call(
                &get_path(&created_groups[0].id, &created_user.id, ""),
                SCOPE,
                get_group_permissions_from_user,
                test_app.valid_authorizate(TestRequest::get(), &me.id),
            )
            .await;
        assert_eq!(resp.status(), http::StatusCode::FORBIDDEN);

        test_app.app_state.pgdb.clear_db();
    }

    /// # Test: `test_other_permissions_successfully`
    ///
    /// Validates the behavior of the `get_group_permissions_from_user` handler in Actix-Web when a user successfully retrieves the permissions of another user for a specific group. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates two test users: `me` (an authorized user attempting to retrieve permissions) and `created_user` (the user whose permissions are being queried).
    ///    - Creates an example group (`created_groups`) associated with `created_user`.
    ///    - Creates resource permissions for both the "Group" and "Permission" resources (`permissions`), allowing read access and other permissions.
    ///    - Creates user permissions for `created_user`, granting read access to the "Group" resource within the specified group, and for `me`, granting read access and other permissions to the "Permission" resource globally.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `get_group_permissions_from_user` using the test application instance with a valid access token for `me`. `me` has the required permissions to query `created_user`'s group permissions.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "OK" (200), indicating a successful request.
    ///    - Validates the response body, ensuring that it contains the expected group permissions for `created_user`.
    ///    - Ensures that the test environment is cleaned up by clearing the database.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_other_permissions_successfully() {
        use crate::handlers::groups::group_id::users::user_id::user_id::get_group_permissions_from_user;

        let test_app = TestRepo::new().await;

        let me = util::create_other_test_user(&test_app.user_repo);

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
            }, vec![AccessType::Read, AccessType::Other])],
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
                user_id: me.id,
                group_id: None,
                ressource: "permission".to_string()
            },
            vec![OptionalUserAccessType {
                access_type: AccessType::Read,
                permission: Some(true),
                set_permission: None,
                set_set_permission: None,
            }, OptionalUserAccessType {
                access_type: AccessType::Other,
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
                &get_path(&created_groups[0].id, &created_user.id, ""),
                SCOPE,
                get_group_permissions_from_user,
                test_app.valid_authorizate(TestRequest::get(), &me.id),
            )
            .await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let permission_response: PermissionListResponseWithCount = test::read_body_json(resp).await;
        util::assert_permission_response(&permission_response.permission_list, &vec![user_permission_list[0].clone()]);
        test_app.app_state.pgdb.clear_db();
    }

    /// # Test: `test_my_permissions_successfully_group_only`
    ///
    /// Validates the behavior of the `get_group_permissions_from_user` handler in Actix-Web when a user successfully retrieves only group permissions for another user. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a test user (`created_user`) and an example group (`created_groups`) associated with `created_user`.
    ///    - Creates resource permissions for both the "Group" and "Permission" resources (`permissions`), allowing read access.
    ///    - Creates user permissions for `created_user`, granting read access to the "Group" resource within the specified group and read access to the "Permission" resource globally.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `get_group_permissions_from_user` using the test application instance with a valid access token for `created_user`. The query parameter `group_only` is set to true.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "OK" (200), indicating a successful request.
    ///    - Validates the response body, ensuring that it contains the expected group permissions for `created_user`.
    ///    - Ensures that the test environment is cleaned up by clearing the database.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_my_permissions_successfully_group_only() {
        use crate::handlers::groups::group_id::users::user_id::user_id::get_group_permissions_from_user;

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
                &get_path(&created_groups[0].id, &created_user.id, "?group_only=true"),
                SCOPE,
                get_group_permissions_from_user,
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
    /// Validates the behavior of the `get_group_permissions_from_user` handler in Actix-Web when a user successfully retrieves paginated permissions. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a test user (`created_user`) and an example group (`created_groups`) associated with `created_user`.
    ///    - Creates resource permissions for the "Group," "Permission," and "Answer" resources (`permissions`), allowing read access.
    ///    - Creates user permissions for `created_user`, granting read access to the "Group" and "Permission" resources globally and read access to the "Answer" resource within the specified group.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `get_group_permissions_from_user` using the test application instance with a valid access token for `created_user`. The query parameters `page` is set to 1, and `limit` is set to 2.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "OK" (200), indicating a successful request.
    ///    - Validates the response body, ensuring that it contains the expected paginated permissions for `created_user`.
    ///    - Ensures that the test environment is cleaned up by clearing the database.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_paging_my_permissions_successfully() {
        use crate::handlers::groups::group_id::users::user_id::user_id::get_group_permissions_from_user;

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
                &get_path(&created_groups[0].id, &created_user.id, "?page=1&limit=2"),
                SCOPE,
                get_group_permissions_from_user,
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
    /// Validates the behavior of the `get_group_permissions_from_user` handler in Actix-Web when a user successfully retrieves paginated permissions restricted to groups only. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a test user (`created_user`) and an example group (`created_groups`) associated with `created_user`.
    ///    - Creates resource permissions for the "Group," "Permission," "Answer," "a," and "b" resources (`permissions`), allowing read access.
    ///    - Creates user permissions for `created_user`, granting read access to the "Group" and "Permission" resources globally, read access to the "Answer" resource within the specified group, and read access to the "a" and "b" resources within the group.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `get_group_permissions_from_user` using the test application instance with a valid access token for `created_user`. The query parameters `page` is set to 1, `limit` is set to 2, and `group_only` is set to true.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "OK" (200), indicating a successful request.
    ///    - Validates the response body, ensuring that it contains the expected paginated permissions for `created_user` restricted to groups only.
    ///    - Ensures that the test environment is cleaned up by clearing the database.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_paging_my_permissions_successfully_groups_only() {
        use crate::handlers::groups::group_id::users::user_id::user_id::get_group_permissions_from_user;

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
                &get_path(&created_groups[0].id, &created_user.id, "?page=1&limit=2&group_only=true"),
                SCOPE,
                get_group_permissions_from_user,
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
    /// Validates the behavior of the `get_group_permissions_from_user` handler in Actix-Web when a user successfully retrieves permissions for a specific list of resources. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a test user (`created_user`) and an example group (`created_groups`) associated with `created_user`.
    ///    - Creates resource permissions for the "Group," "Permission," and "Answer" resources (`permissions`), allowing read access.
    ///    - Creates user permissions for `created_user`, granting read access to the "Group" and "Permission" resources globally, and read access to the "Answer" resource within the specified group.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `get_group_permissions_from_user` using the test application instance with a valid access token for `created_user`. The query parameters include `ressources[]` with the names of the "Group" and "Permission" resources.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "OK" (200), indicating a successful request.
    ///    - Validates the response body, ensuring that it contains the expected permissions for `created_user` for the specified list of resources.
    ///    - Ensures that the test environment is cleaned up by clearing the database.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_my_ressource_list_successfully() {
        use crate::handlers::groups::group_id::users::user_id::user_id::get_group_permissions_from_user;

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
                &get_path(&created_groups[0].id, &created_user.id, &format!("?ressources[]={},{}", user_permission_list[0].0.ressource, user_permission_list[1].0.ressource)),
                SCOPE,
                get_group_permissions_from_user,
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
    /// Validates the behavior of the `get_group_permissions_from_user` handler in Actix-Web when a user successfully retrieves group-only permissions for a specific list of resources. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a test user (`created_user`) and an example group (`created_groups`) associated with `created_user`.
    ///    - Creates resource permissions for the "Group," "Permission," and "Answer" resources (`permissions`), allowing read access.
    ///    - Creates user permissions for `created_user`, granting read access to the "Group" and "Permission" resources globally, and read access to the "Answer" resource within the specified group.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `get_group_permissions_from_user` using the test application instance with a valid access token for `created_user`. The query parameters include `ressources[]` with the names of the "Group" and "Answer" resources and `group_only=true`.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "OK" (200), indicating a successful request.
    ///    - Validates the response body, ensuring that it contains the expected group-only permissions for `created_user` for the specified list of resources.
    ///    - Ensures that the test environment is cleaned up by clearing the database.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_my_ressource_list_successfully_group_only() {
        use crate::handlers::groups::group_id::users::user_id::user_id::get_group_permissions_from_user;

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
                &get_path(&created_groups[0].id, &created_user.id, &format!("?ressources[]={},{}&group_only=true", user_permission_list[0].0.ressource, user_permission_list[2].0.ressource)),
                SCOPE,
                get_group_permissions_from_user,
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
    /// Validates the behavior of the `get_group_permissions_from_user` handler in Actix-Web when a user successfully retrieves a paginated list of permissions for a specific set of resources. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a test user (`created_user`) and an example group (`created_groups`) associated with `created_user`.
    ///    - Creates resource permissions for the "Group," "Permission," and "Answer" resources (`permissions`), allowing read access.
    ///    - Creates user permissions for `created_user`, granting read access to the "Group" and "Permission" resources globally, and read access to the "Answer" resource within the specified group.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `get_group_permissions_from_user` using the test application instance with a valid access token for `created_user`. The query parameters include `ressources[]` with the names of the "Group" and "Permission" resources, and `page=1&limit=1`.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "OK" (200), indicating a successful request.
    ///    - Validates the response body, ensuring that it contains the expected paginated permissions for `created_user` for the specified set of resources.
    ///    - Ensures that the test environment is cleaned up by clearing the database.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_my_paging_ressource_list_successfully() {
        use crate::handlers::groups::group_id::users::user_id::user_id::get_group_permissions_from_user;

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
                &get_path(&created_groups[0].id, &created_user.id, &format!("?ressources[]={},{}&page=1&limit=1", user_permission_list[0].0.ressource, user_permission_list[1].0.ressource)),
                SCOPE,
                get_group_permissions_from_user,
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
    /// Validates the behavior of the `get_group_permissions_from_user` handler in Actix-Web when a user successfully retrieves a paginated list of permissions for a specific set of resources within a group. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a test user (`created_user`) and an example group (`created_groups`) associated with `created_user`.
    ///    - Creates resource permissions for the "Group," "Permission," "Answer," "A," and "B" resources (`permissions`), allowing read access.
    ///    - Creates user permissions for `created_user`, granting read access to the "Group" and "Permission" resources globally, and read access to the "Answer," "A," and "B" resources within the specified group.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `get_group_permissions_from_user` using the test application instance with a valid access token for `created_user`. The query parameters include `ressources[]` with the names of the "Group" and "Permission" resources, and `page=1&limit=1&group_only=true`.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "OK" (200), indicating a successful request.
    ///    - Validates the response body, ensuring that it contains an empty list of permissions since the query parameters specify `group_only=true`.
    ///    - Ensures that the test environment is cleaned up by clearing the database.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_my_paging_ressource_list_successfully_group_only() {
        use crate::handlers::groups::group_id::users::user_id::user_id::get_group_permissions_from_user;

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
                &get_path(&created_groups[0].id, &created_user.id, &format!("?ressources[]={},{}&page=1&limit=1&group_only=true", user_permission_list[0].0.ressource, user_permission_list[1].0.ressource)),
                SCOPE,
                get_group_permissions_from_user,
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
    /// Validates the behavior of the `get_group_permissions_from_user` handler in Actix-Web when an invalid access token is provided. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a test user (`created_user`) and an example group (`created_groups`) associated with `created_user`.
    ///    - Creates resource permissions for the "Group" and "Permission" resources (`permissions`), allowing read access.
    ///    - Creates user permissions for `created_user`, granting read access to the "Group" and "Permission" resources globally.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `get_group_permissions_from_user` using the test application instance with an invalid access token for `created_user`.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "UNAUTHORIZED" (401), indicating that the access token is invalid.
    ///    - Ensures that the test environment is cleaned up by clearing the database.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_permissions_invalid_access_token() {
        use crate::handlers::groups::group_id::users::user_id::user_id::get_group_permissions_from_user;

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
                &get_path(&created_groups[0].id, &created_user.id, ""),
                SCOPE,
                get_group_permissions_from_user,
                test_app.invalid_authorizate(TestRequest::get(), &created_user.id),
            )
            .await;
        assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);
        test_app.app_state.pgdb.clear_db();
    }

    /// # Test: `test_permissions_no_permission`
    ///
    /// Validates the behavior of the `get_group_permissions_from_user` handler in Actix-Web when a user has no permission for the requested resource. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a test user (`created_user`) and an example group (`created_groups`) associated with `created_user`.
    ///    - Creates resource permissions for the "Group" and "Permission" resources (`permissions`), allowing read access.
    ///    - Creates user permissions for `created_user`, granting read access only to the "Group" resource.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `get_group_permissions_from_user` using the test application instance with a valid access token for `created_user`.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "FORBIDDEN" (403), indicating that the user does not have permission for the requested resource.
    ///    - Ensures that the test environment is cleaned up by clearing the database.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_permissions_no_permission() {
        use crate::handlers::groups::group_id::users::user_id::user_id::get_group_permissions_from_user;

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
                &get_path(&created_groups[0].id, &created_user.id, ""),
                SCOPE,
                get_group_permissions_from_user,
                test_app.valid_authorizate(TestRequest::get(), &created_user.id),
            )
            .await;
        assert_eq!(resp.status(), http::StatusCode::FORBIDDEN);
        test_app.app_state.pgdb.clear_db();
    }
}