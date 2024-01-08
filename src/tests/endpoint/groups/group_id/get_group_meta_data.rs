//docu written with chat-gpt
#[cfg(test)]
mod get_group_meta_data_tests {
    use actix_web::{test::{TestRequest, self}, http};
    use serde_json::Value;
    use uuid::Uuid;

    use crate::{tests::{util, test::TestRepo}, models::{permissions::{NewRessource, OptionalUserAccessType}, groups::{NewUserPermission, CreateGroupResponse, GroupInfoResponse, GroupMetaDataResponse}, util::{AccessType, PagingSchema, OrderDir}, task::TaskPagingResponse, roles::{NewRole, UpdateRolePermission, NewRolePermission, NewRoleAccessType, UpdateRoleAccesType}}};

    static SCOPE: &'static str = "/api/groups/{group_id}"; 
    
    fn get_path(group_id: &Uuid) -> String {
        format!("/api/groups/{}/metadata", group_id)
    }

    /// # Test: `test_get_group_meta_data`
    ///
    /// Validates the behavior of the `get_group_meta_data` handler in Actix-Web when a user attempts to retrieve metadata about a group. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a standard test user (`created_user`).
    ///    - Creates a resource permission for the "Group Meta Data" resource (`permissions`).
    ///    - Creates an example group and assigns it to the user (`created_groups`).
    ///
    /// 2. Execution:
    ///    - Makes an API call to `get_group_meta_data` using the test application instance with a valid access token for the user. The user has read permissions for the "Group Meta Data" resource.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "OK" (200), indicating a successful request.
    ///    - Verifies that the response body contains the correct metadata information about the group, including the group ID, creation user ID (`created_from`), and update user ID (`updated_from`).
    ///    - Additional TODO: Test dates (Add specific assertions for date comparisons if applicable).
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_get_group_meta_data() {
        use crate::handlers::groups::group_id::group_id::get_group_meta_data;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_groups = util::create_example_groups(&test_app.group_repo, 1, created_user.id);

        let permissions = util::create_ressource(
            &test_app.permission_repo,
            &vec![(NewRessource {
                key_name: &"Group".to_string(),
                key_value: &"group_meta_data".to_string(),
            }, vec![AccessType::Read])
            ],
        );

        let user_permission_list: Vec<(NewUserPermission, Vec<OptionalUserAccessType>)> = vec![(NewUserPermission {
                user_id: created_user.id,
                group_id: None,
                ressource: "group_meta_data".to_string()
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
            get_group_meta_data,
            test_app.valid_authorizate(TestRequest::get(), &created_user.id),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let response: GroupMetaDataResponse = test::read_body_json(resp).await;

        assert_eq!(response.id, created_groups[0].id);
        assert_eq!(response.created_from, created_user.id);
        assert_eq!(response.updated_from, created_user.id);
        // TODO: Test dates
    }

    /// # Test: `test_get_group_meta_data_group_permission`
    ///
    /// Validates the behavior of the `get_group_meta_data` handler in Actix-Web when a user attempts to retrieve metadata about a group with specific group-level permissions. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a standard test user (`created_user`).
    ///    - Creates a resource permission for the "Group Meta Data" resource (`permissions`).
    ///    - Creates an example group and assigns it to the user (`created_groups`).
    ///    - Assigns read permission for the "Group Meta Data" resource to the user for the specific group.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `get_group_meta_data` using the test application instance with a valid access token for the user. The user has read permissions for the "Group Meta Data" resource and specific group-level permissions for the group.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "OK" (200), indicating a successful request.
    ///    - Verifies that the response body contains the correct metadata information about the group, including the group ID, creation user ID (`created_from`), and update user ID (`updated_from`).
    ///    - Additional TODO: Test dates (Add specific assertions for date comparisons if applicable).
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_get_group_meta_data_group_permission() {
        use crate::handlers::groups::group_id::group_id::get_group_meta_data;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_groups = util::create_example_groups(&test_app.group_repo, 1, created_user.id);

        let permissions = util::create_ressource(
            &test_app.permission_repo,
            &vec![(NewRessource {
                key_name: &"Group".to_string(),
                key_value: &"group_meta_data".to_string(),
            }, vec![AccessType::Create, AccessType::Read])
            ],
        );

        let user_permission_list: Vec<(NewUserPermission, Vec<OptionalUserAccessType>)> = vec![(NewUserPermission {
                user_id: created_user.id,
                group_id: Some(created_groups[0].id),
                ressource: "group_meta_data".to_string()
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
            get_group_meta_data,
            test_app.valid_authorizate(TestRequest::get(), &created_user.id),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let response: GroupMetaDataResponse = test::read_body_json(resp).await;

        assert_eq!(response.id, created_groups[0].id);
        assert_eq!(response.created_from, created_user.id);
        assert_eq!(response.updated_from, created_user.id);
        // TODO: Test dates
    }

    /// # Test: `test_get_not_existing_group`
    ///
    /// Validates the behavior of the `get_group_meta_data` handler in Actix-Web when a user attempts to retrieve metadata about a non-existing group. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a standard test user (`created_user`).
    ///    - Creates a resource permission for the "Group Meta Data" resource (`permissions`).
    ///    - Assigns read permission for the "Group Meta Data" resource to the user at the global level.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `get_group_meta_data` using the test application instance with a valid access token for the user. The user has read permissions for the "Group Meta Data" resource globally.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "Not Found" (404), indicating that the requested group does not exist.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_get_not_existing_group() {
        use crate::handlers::groups::group_id::group_id::get_group_meta_data;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let permissions = util::create_ressource(
            &test_app.permission_repo,
            &vec![(NewRessource {
                key_name: &"Group".to_string(),
                key_value: &"group_meta_data".to_string(),
            }, vec![AccessType::Create, AccessType::Read])
            ],
        );

        let user_permission_list: Vec<(NewUserPermission, Vec<OptionalUserAccessType>)> = vec![(NewUserPermission {
                user_id: created_user.id,
                group_id: None,
                ressource: "group_meta_data".to_string()
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
            get_group_meta_data,
            test_app.valid_authorizate(TestRequest::get(), &created_user.id),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::NOT_FOUND);
    }

    /// # Test: `test_get_group_no_permission`
    ///
    /// Validates the behavior of the `get_group_meta_data` handler in Actix-Web when a user attempts to retrieve metadata about a group for which they do not have permission. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a standard test user (`created_user`).
    ///    - Creates an example group (`created_groups`) associated with the user.
    ///    - Creates a resource permission for the "Group Meta Data" resource (`permissions`), allowing both create and read access.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `get_group_meta_data` using the test application instance with a valid access token for the user. The user does not have read permission for the "Group Meta Data" resource associated with the group.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "Forbidden" (403), indicating that the user does not have permission to retrieve metadata for the specified group.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_get_group_no_permission() {
        use crate::handlers::groups::group_id::group_id::get_group_meta_data;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_groups = util::create_example_groups(&test_app.group_repo, 1, created_user.id);

        let permissions = util::create_ressource(
            &test_app.permission_repo,
            &vec![(NewRessource {
                key_name: &"Group".to_string(),
                key_value: &"group_meta_data".to_string(),
            }, vec![AccessType::Create, AccessType::Read])
            ],
        );

        let resp = test_app
        .call(
            &get_path(&created_groups[0].id),
            SCOPE,
            get_group_meta_data,
            test_app.valid_authorizate(TestRequest::get(), &created_user.id),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::FORBIDDEN);
    }
}