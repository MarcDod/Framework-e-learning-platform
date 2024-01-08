//docu written with chat-gpt
#[cfg(test)]
mod add_member_to_group_tests {
    use actix_web::{test::{TestRequest, self}, http};
    use serde_json::Value;
    use uuid::Uuid;

    use crate::{tests::{util, test::TestRepo}, models::{permissions::{NewRessource, OptionalUserAccessType}, groups::{NewUserPermission, CreateGroupResponse}, util::{AccessType, PagingSchema, OrderDir}, task::TaskPagingResponse, roles::{NewRole, UpdateRolePermission, NewRolePermission, NewRoleAccessType, UpdateRoleAccesType}, members::{MemberInfoResponse, MemberListResponse}}};

    static SCOPE: &'static str = "/api/groups/{group_id}/members"; 
    
    fn get_path(group_id: &Uuid) -> String {
        format!("/api/groups/{}/members/", group_id)
    }

    fn create_body(email: String) -> Value {
        serde_json::json!({
            "new_members": [email],
        })
    }

    fn create_body_and_wrong(email: String) -> Value {
        serde_json::json!({
            "new_members": [email, "ha"],
        })
    }

    /// # Test: `test_add_member`
    ///
    /// Validates the behavior of the `add_member_to_group` handler in Actix-Web when adding a member to a group. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a test user (`created_user`).
    ///    - Creates an example group (`created_groups`) associated with `created_user`.
    ///    - Sets up required permissions for adding a member (`add_member`) to the group.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `add_member_to_group` using the test application instance with a valid authorization token for `created_user`.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "CREATED" (201), indicating successful member addition.
    ///    - Reads the response body and asserts that it contains the added member information.
    ///    - Retrieves and verifies the user's permissions for the group from the database.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_add_member() {
        use crate::handlers::groups::group_id::members::members::add_member_to_group;
     
        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_groups = util::create_example_groups(&test_app.group_repo, 1, created_user.id);

        util::create_permissions(&test_app.permission_repo, &test_app.group_repo, "group_member", &created_user.id, &AccessType::Write);
        util::create_permissions(&test_app.permission_repo, &test_app.group_repo, "group", &created_user.id, &AccessType::Read);

        util::create_role(&test_app.permission_repo, &vec![NewRole {
            name: &"Created Group".to_string(),
            value_key: &"add_member".to_string(),
        }]);

        util::update_role(&test_app.permission_repo, &UpdateRolePermission {
            role_permission: NewRolePermission {
                ressource: "group".to_string(),
                role: "add_member".to_string(),
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
            &get_path(&created_groups[0].id),
            SCOPE,
            add_member_to_group,
            test_app.valid_authorizate(TestRequest::post().set_json(create_body(created_user.email)), &created_user.id),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::CREATED);

        let response: MemberListResponse = test::read_body_json(resp).await;

        let permissions = test_app.permission_repo
        .fetch_user_permissions(&created_user.id, &PagingSchema {
            limit: 200,
            page: 0,
            order: OrderDir::DESC,
        }, &Some(created_groups[0].id), true, &None).unwrap();

        assert_eq!(response.members.len(), 1);
        assert_eq!(response.members[0].user.id, created_user.id);
        assert_eq!(permissions.total_count, 1);
        assert_eq!(permissions.permission_list.len(), 1);
        assert_eq!(permissions.permission_list[0].access_types.len(), 1);
        assert_eq!(permissions.permission_list[0].key_value, "group");
        assert_eq!(permissions.permission_list[0].access_types[0].access_type, AccessType::Read);
        assert_eq!(permissions.permission_list[0].access_types[0].permission, true);
        assert_eq!(permissions.permission_list[0].access_types[0].set_permission, false);
        assert_eq!(permissions.permission_list[0].access_types[0].set_set_permission, false);

    }

    /// # Test: `test_add_member_and_wrong`
    ///
    /// Validates the behavior of the `add_member_to_group` handler in Actix-Web when attempting to add a member to a group with incorrect input. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a test user (`created_user`).
    ///    - Creates an example group (`created_groups`) associated with `created_user`.
    ///    - Sets up required permissions for adding a member (`add_member`) to the group.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `add_member_to_group` using the test application instance with a valid authorization token for `created_user` and incorrect input for the new member.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "CREATED" (201), indicating successful member addition.
    ///    - Reads the response body and asserts that it contains the added member information.
    ///    - Retrieves and verifies the user's permissions for the group from the database.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_add_member_and_wrong() {
        use crate::handlers::groups::group_id::members::members::add_member_to_group;
     
        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_groups = util::create_example_groups(&test_app.group_repo, 1, created_user.id);

        util::create_permissions(&test_app.permission_repo, &test_app.group_repo, "group_member", &created_user.id, &AccessType::Write);
        util::create_permissions(&test_app.permission_repo, &test_app.group_repo, "group", &created_user.id, &AccessType::Read);

        util::create_role(&test_app.permission_repo, &vec![NewRole {
            name: &"Created Group".to_string(),
            value_key: &"add_member".to_string(),
        }]);

        util::update_role(&test_app.permission_repo, &UpdateRolePermission {
            role_permission: NewRolePermission {
                ressource: "group".to_string(),
                role: "add_member".to_string(),
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
            &get_path(&created_groups[0].id), 
            SCOPE,
            add_member_to_group,
            test_app.valid_authorizate(TestRequest::post().set_json(create_body_and_wrong(created_user.email)), &created_user.id),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::CREATED);

        let response: MemberListResponse = test::read_body_json(resp).await;

        let permissions = test_app.permission_repo
        .fetch_user_permissions(&created_user.id, &PagingSchema {
            limit: 200,
            page: 0,
            order: OrderDir::DESC,
        }, &Some(created_groups[0].id), true, &None).unwrap();

        assert_eq!(response.members.len(), 1);
        assert_eq!(response.members[0].user.id, created_user.id);
        assert_eq!(permissions.total_count, 1);
        assert_eq!(permissions.permission_list.len(), 1);
        assert_eq!(permissions.permission_list[0].access_types.len(), 1);
        assert_eq!(permissions.permission_list[0].key_value, "group");
        assert_eq!(permissions.permission_list[0].access_types[0].access_type, AccessType::Read);
        assert_eq!(permissions.permission_list[0].access_types[0].permission, true);
        assert_eq!(permissions.permission_list[0].access_types[0].set_permission, false);
        assert_eq!(permissions.permission_list[0].access_types[0].set_set_permission, false);

    }

    /// # Test: `test_add_member_no_auth`
    ///
    /// Validates the behavior of the `add_member_to_group` handler in Actix-Web when attempting to add a member to a group without proper authorization. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a test user (`created_user`).
    ///    - Creates an example group (`created_groups`) associated with `created_user`.
    ///    - Sets up required permissions for adding a member (`add_member`) to the group.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `add_member_to_group` using the test application instance with an invalid authorization token for `created_user`.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "UNAUTHORIZED" (401), indicating insufficient authorization.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_add_member_no_auth() {
        use crate::handlers::groups::group_id::members::members::add_member_to_group;
     
        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_groups = util::create_example_groups(&test_app.group_repo, 1, created_user.id);

        util::create_permissions(&test_app.permission_repo, &test_app.group_repo, "group_member", &created_user.id, &AccessType::Write);
        util::create_permissions(&test_app.permission_repo, &test_app.group_repo, "group", &created_user.id, &AccessType::Read);

        util::create_role(&test_app.permission_repo, &vec![NewRole {
            name: &"Created Group".to_string(),
            value_key: &"add_member".to_string(),
        }]);

        util::update_role(&test_app.permission_repo, &UpdateRolePermission {
            role_permission: NewRolePermission {
                ressource: "group".to_string(),
                role: "add_member".to_string(),
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
            &get_path(&created_groups[0].id),
            SCOPE,
            add_member_to_group,
            test_app.invalid_authorizate(TestRequest::post().set_json(create_body(created_user.email)), &created_user.id),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);

    }

    /// # Test: `test_add_member_no_permission`
    ///
    /// Validates the behavior of the `add_member_to_group` handler in Actix-Web when attempting to add a member to a group without the required permission. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a test user (`created_user`).
    ///    - Creates an example group (`created_groups`) associated with `created_user`.
    ///    - Sets up permissions for reading the group (`group` permission) but not for adding a member.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `add_member_to_group` using the test application instance with valid authorization token for `created_user`.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "FORBIDDEN" (403), indicating insufficient permission to add a member.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_add_member_no_permission() {
        use crate::handlers::groups::group_id::members::members::add_member_to_group;
     
        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_groups = util::create_example_groups(&test_app.group_repo, 1, created_user.id);

        util::create_permissions(&test_app.permission_repo, &test_app.group_repo, "group", &created_user.id, &AccessType::Read);

        util::create_role(&test_app.permission_repo, &vec![NewRole {
            name: &"Created Group".to_string(),
            value_key: &"add_member".to_string(),
        }]);

        util::update_role(&test_app.permission_repo, &UpdateRolePermission {
            role_permission: NewRolePermission {
                ressource: "group".to_string(),
                role: "add_member".to_string(),
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
            &get_path(&created_groups[0].id),
            SCOPE,
            add_member_to_group,
            test_app.valid_authorizate(TestRequest::post().set_json(create_body(created_user.email)), &created_user.id),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::FORBIDDEN);

    }
}