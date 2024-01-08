//docu written with chat-gpt
#[cfg(test)]
mod get_group_tests {
    use actix_web::{test::{TestRequest, self}, http};
    use serde_json::Value;
    use uuid::Uuid;

    use crate::{tests::{util, test::TestRepo}, models::{permissions::{NewRessource, OptionalUserAccessType}, groups::{NewUserPermission, CreateGroupResponse, GroupInfoResponse}, util::{AccessType, PagingSchema, OrderDir}, task::TaskPagingResponse, roles::{NewRole, UpdateRolePermission, NewRolePermission, NewRoleAccessType, UpdateRoleAccesType}}};

    static SCOPE: &'static str = "/api/groups/{group_id}"; 
    
    fn get_path(group_id: &Uuid) -> String {
        format!("/api/groups/{}/", group_id)
    }

    /// # Test: `test_get_group`
    ///
    /// Validates the behavior of the `get_group` handler in Actix-Web when a user attempts to retrieve information about a specific group. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a standard test user (`created_user`).
    ///    - Creates an example group (`created_groups`) for testing.
    ///    - Creates a resource permission for the "Group" resource (`permissions`).
    ///
    /// 2. Execution:
    ///    - Makes an API call to `get_group` using the test application instance with a valid access token for the user. The user has read permissions for the "Group" resource.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "OK" (200), indicating a successful request.
    ///    - Reads the group information from the response and asserts that it matches the details of the created group.
    ///
    /// 4. Cleanup:
    ///    - Clears the database after the test is completed.
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_get_group() {
        use crate::handlers::groups::group_id::group_id::get_group;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_groups = util::create_example_groups(&test_app.group_repo, 1, created_user.id);

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
            &get_path(&created_groups[0].id),
            SCOPE,
            get_group,
            test_app.valid_authorizate(TestRequest::get(), &created_user.id),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let response: GroupInfoResponse = test::read_body_json(resp).await;

        assert_eq!(response.id, created_groups[0].id);
        assert_eq!(response.name, created_groups[0].name);
        assert_eq!(response.parent, created_groups[0].parent);
    }

    /// # Test: `test_get_group_group_permission`
    ///
    /// Validates the behavior of the `get_group` handler in Actix-Web when a user attempts to retrieve information about a specific group with group-level permission. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a standard test user (`created_user`).
    ///    - Creates an example group (`created_groups`) for testing.
    ///    - Creates a resource permission for the "Group" resource (`permissions`).
    ///    - Assigns a user permission specifically for the created group, allowing read access (`user_permission_list`).
    ///
    /// 2. Execution:
    ///    - Makes an API call to `get_group` using the test application instance with a valid access token for the user. The user has group-level read permissions for the "Group" resource.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "OK" (200), indicating a successful request.
    ///    - Reads the group information from the response and asserts that it matches the details of the created group.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_get_group_group_permission() {
        use crate::handlers::groups::group_id::group_id::get_group;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_groups = util::create_example_groups(&test_app.group_repo, 1, created_user.id);

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
                group_id: Some(created_groups[0].id),
                ressource: "group".to_string()
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
            &get_path(&created_groups[0].id),
            SCOPE,
            get_group,
            test_app.valid_authorizate(TestRequest::get(), &created_user.id),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let response: GroupInfoResponse = test::read_body_json(resp).await;

        assert_eq!(response.id, created_groups[0].id);
        assert_eq!(response.name, created_groups[0].name);
        assert_eq!(response.parent, created_groups[0].parent);
    }

    /// # Test: `test_get_not_existing_group`
    ///
    /// Validates the behavior of the `get_group` handler in Actix-Web when a user attempts to retrieve information about a non-existing group. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a standard test user (`created_user`).
    ///    - Creates a resource permission for the "Group" resource (`permissions`).
    ///    - Assigns a user permission allowing read access to the "Group" resource (`user_permission_list`).
    ///
    /// 2. Execution:
    ///    - Makes an API call to `get_group` using the test application instance with a valid access token for the user. The user has general read permissions for the "Group" resource.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "Not Found" (404), indicating that the requested group was not found.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_get_not_existing_group() {
        use crate::handlers::groups::group_id::group_id::get_group;

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
            &get_path(&Uuid::new_v4()),
            SCOPE,
            get_group,
            test_app.valid_authorizate(TestRequest::get(), &created_user.id),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::NOT_FOUND);
    }

    /// # Test: `test_get_group_no_permission`
    ///
    /// Validates the behavior of the `get_group` handler in Actix-Web when a user attempts to retrieve information about a group without the necessary permissions. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a standard test user (`created_user`).
    ///    - Creates a resource permission for the "Group" resource (`permissions`).
    ///    - Creates an example group and assigns it to the user (`created_groups`).
    ///
    /// 2. Execution:
    ///    - Makes an API call to `get_group` using the test application instance with a valid access token for the user. The user lacks read permissions for the "Group" resource.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "Forbidden" (403), indicating that the user does not have the necessary permissions to retrieve information about the group.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_get_group_no_permission() {
        use crate::handlers::groups::group_id::group_id::get_group;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_groups = util::create_example_groups(&test_app.group_repo, 1, created_user.id);

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
            &get_path(&created_groups[0].id),
            SCOPE,
            get_group,
            test_app.valid_authorizate(TestRequest::get(), &created_user.id),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::FORBIDDEN);
    }
}