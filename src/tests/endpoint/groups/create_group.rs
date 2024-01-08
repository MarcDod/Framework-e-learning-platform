//docu written with chat-gpt
#[cfg(test)]
mod create_groups_tests {
    use actix_web::{test::{TestRequest, self}, http};
    use serde_json::Value;
    use uuid::Uuid;

    use crate::{tests::{util, test::TestRepo}, models::{permissions::{NewRessource, OptionalUserAccessType}, groups::{NewUserPermission, CreateGroupResponse}, util::{AccessType, PagingSchema, OrderDir}, task::TaskPagingResponse, roles::{NewRole, UpdateRolePermission, NewRolePermission, NewRoleAccessType, UpdateRoleAccesType}}};

    static SCOPE: &'static str = "/api/groups"; 
    
    fn get_path() -> String {
        format!("/api/groups/")
    }

    fn create_body() -> Value {
        serde_json::json!({
            "name": "Test",
        })
    }

    fn create_illegal_body() -> Value {
        serde_json::json!({
            "name": "Test",
            "parent": Uuid::new_v4().to_string()
        })
    }

    /// # Test: `test_create_groups`
    ///
    /// Validates the behavior of the `create_group` handler in Actix-Web when a user attempts to create a new group. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a standard test user (`created_user`).
    ///    - Creates a resource permission for the "Group" resource (`permissions`).
    ///    - Assigns create and read permissions to the user for the "Group" resource.
    ///    - Creates a role named "Created Group" for testing.
    ///    - Updates the role to grant read permission for the "Group" resource.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `create_group` using the test application instance with a valid access token for the user.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "Created" (201), indicating that the group has been successfully created.
    ///    - Reads the response body to get the details of the created group (`response`).
    ///    - Fetches the list of active groups from the database.
    ///    - Fetches the user's permissions for the created group.
    ///    - Asserts that the total count and number of groups match expectations.
    ///    - Asserts that the details of the created group match the response.
    ///    - Asserts that the user has read permission for the created group.
    ///
    /// 4. Cleanup:
    ///    - Clears the database after the test is completed.
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_create_groups() {
        use crate::handlers::groups::groups::create_group;
     
        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let permissions = util::create_ressource(
            &test_app.permission_repo,
            &vec![(NewRessource {
                key_name: &"Group".to_string(),
                key_value: &"group".to_string(),
            }, vec![AccessType::Create, AccessType::Read])
            ],
        );

        let user_permission_list: Vec<(NewUserPermission, Vec<OptionalUserAccessType>)> = vec![(NewUserPermission {
                user_id: created_user.id,
                group_id: None,
                ressource: "group".to_string()
            }, 
            vec![OptionalUserAccessType {
                access_type: AccessType::Create,
                permission: Some(true),
                set_permission: None,
                set_set_permission: None,
            }])
        ];

        util::create_permissions_for_user(
            &test_app.group_repo,
            &user_permission_list,
        );

        util::create_role(&test_app.permission_repo, &vec![NewRole {
            name: &"Created Group".to_string(),
            value_key: &"created_group".to_string(),
        }]);

        util::update_role(&test_app.permission_repo, &UpdateRolePermission {
            role_permission: NewRolePermission {
                ressource: "group".to_string(),
                role: "created_group".to_string(),
            },
            role_access_types: vec![
                UpdateRoleAccesType {
                    access_type: AccessType::Read,
                    permission: Some(true),
                    set_permission: None,
                    set_set_permission: None,
                }
            ]
        });

        let resp = test_app
        .call(
            &get_path(),
            SCOPE,
            create_group,
            test_app.valid_authorizate(TestRequest::post().set_json(create_body()), &created_user.id),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::CREATED);

        let response: CreateGroupResponse = test::read_body_json(resp).await;

        let groups = test_app.group_repo.fetch_all_active_groups(&PagingSchema {
            limit: 200,
            page: 0,
            order: OrderDir::DESC,
        }, &None).unwrap();

        let permissions = test_app.permission_repo
        .fetch_user_permissions(&created_user.id, &PagingSchema {
            limit: 200,
            page: 0,
            order: OrderDir::DESC,
        }, &Some(groups.group_info_list[0].id), true, &None).unwrap();

        assert_eq!(groups.total_count, 1);
        assert_eq!(groups.group_info_list.len(), 1);
        assert_eq!(groups.group_info_list[0].name, response.name);
        assert_eq!(groups.group_info_list[0].id, response.id);
        assert_eq!(groups.group_info_list[0].parent, response.parent);
        assert_eq!(permissions.total_count, 1);
        assert_eq!(permissions.permission_list.len(), 1);
        assert_eq!(permissions.permission_list[0].access_types.len(), 1);
        assert_eq!(permissions.permission_list[0].key_value, "group");
        assert_eq!(permissions.permission_list[0].access_types[0].access_type, AccessType::Read);
        assert_eq!(permissions.permission_list[0].access_types[0].permission, true);
        assert_eq!(permissions.permission_list[0].access_types[0].set_permission, false);
        assert_eq!(permissions.permission_list[0].access_types[0].set_set_permission, false);

    }

    /// # Test: `test_create_groups_without_role`
    ///
    /// Validates the behavior of the `create_group` handler in Actix-Web when a user attempts to create a new group without a specific role assigned to it. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a standard test user (`created_user`).
    ///    - Creates a resource permission for the "Group" resource (`permissions`).
    ///    - Assigns create and read permissions to the user for the "Group" resource.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `create_group` using the test application instance with a valid access token for the user.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "Created" (201), indicating that the group has been successfully created.
    ///    - Reads the response body to get the details of the created group (`response`).
    ///    - Fetches the list of active groups from the database.
    ///    - Asserts that the total count and number of groups match expectations.
    ///    - Asserts that the details of the created group match the response.
    ///    - Does not check the user's permissions for the created group since no role is assigned.
    ///
    /// 4. Cleanup:
    ///    - Clears the database after the test is completed.
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_create_groups_without_role() {
        use crate::handlers::groups::groups::create_group;
     
        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let permissions = util::create_ressource(
            &test_app.permission_repo,
            &vec![(NewRessource {
                key_name: &"Group".to_string(),
                key_value: &"group".to_string(),
            }, vec![AccessType::Create, AccessType::Read])
            ],
        );

        let user_permission_list: Vec<(NewUserPermission, Vec<OptionalUserAccessType>)> = vec![(NewUserPermission {
                user_id: created_user.id,
                group_id: None,
                ressource: "group".to_string()
            }, 
            vec![OptionalUserAccessType {
                access_type: AccessType::Create,
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
            &get_path(),
            SCOPE,
            create_group,
            test_app.valid_authorizate(TestRequest::post().set_json(create_body()), &created_user.id),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::CREATED);

        let response: CreateGroupResponse = test::read_body_json(resp).await;

        let groups = test_app.group_repo.fetch_all_active_groups(&PagingSchema {
            limit: 200,
            page: 0,
            order: OrderDir::DESC,
        }, &None).unwrap();

        assert_eq!(groups.total_count, 1);
        assert_eq!(groups.group_info_list.len(), 1);
        assert_eq!(groups.group_info_list[0].name, response.name);
        assert_eq!(groups.group_info_list[0].id, response.id);
        assert_eq!(groups.group_info_list[0].parent, response.parent);

    }

    /// # Test: `test_create_group_no_permission`
    ///
    /// Validates the behavior of the `create_group` handler in Actix-Web when a user attempts to create a new group without the required permissions. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a standard test user (`created_user`).
    ///    - Creates a resource permission for the "Group" resource (`permissions`).
    ///
    /// 2. Execution:
    ///    - Makes an API call to `create_group` using the test application instance with a valid access token for the user. The user lacks the necessary permissions for creating a group.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "Forbidden" (403), indicating that the user does not have the required permissions to create a group.
    ///    - Reads the list of active groups from the database and asserts that no groups were created.
    ///
    /// 4. Cleanup:
    ///    - Clears the database after the test is completed.
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_create_group_no_permission() {
        use crate::handlers::groups::groups::create_group;
     
        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let permissions = util::create_ressource(
            &test_app.permission_repo,
            &vec![(NewRessource {
                key_name: &"Group".to_string(),
                key_value: &"group".to_string(),
            }, vec![AccessType::Create, AccessType::Read])
            ],
        );

        let resp = test_app
        .call(
            &get_path(),
            SCOPE,
            create_group,
            test_app.valid_authorizate(TestRequest::post().set_json(create_body()), &created_user.id),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::FORBIDDEN);

        let groups = test_app.group_repo.fetch_all_active_groups(&PagingSchema {
            limit: 200,
            page: 0,
            order: OrderDir::DESC,
        }, &None).unwrap();

        assert_eq!(groups.total_count, 0);
        assert_eq!(groups.group_info_list.len(), 0);
    }


    // Dosnt work, need to be fixed
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_create_group_with_illegal_parent() {
        use crate::handlers::groups::groups::create_group;
     
        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let permissions = util::create_ressource(
            &test_app.permission_repo,
            &vec![(NewRessource {
                key_name: &"Group".to_string(),
                key_value: &"group".to_string(),
            }, vec![AccessType::Create, AccessType::Read])
            ],
        );

        let user_permission_list: Vec<(NewUserPermission, Vec<OptionalUserAccessType>)> = vec![(NewUserPermission {
                user_id: created_user.id,
                group_id: None,
                ressource: "group".to_string()
            }, 
            vec![OptionalUserAccessType {
                access_type: AccessType::Create,
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
            &get_path(),
            SCOPE,
            create_group,
            test_app.valid_authorizate(TestRequest::post().set_json(create_illegal_body()), &created_user.id),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST);

        let groups = test_app.group_repo.fetch_all_active_groups(&PagingSchema {
            limit: 200,
            page: 0,
            order: OrderDir::DESC,
        }, &None).unwrap();

        assert_eq!(groups.total_count, 0);
        assert_eq!(groups.group_info_list.len(), 0);
    }
}