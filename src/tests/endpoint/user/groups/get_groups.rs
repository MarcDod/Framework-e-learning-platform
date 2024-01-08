#[cfg(test)]
mod groups_tests {
    use actix_web::{
        http,
        test::{self, TestRequest},
    };
    use uuid::Uuid;

    use crate::{
        models::{
            groups::{GroupInfoResponse, GroupPagingResponse, NewGroup, NewUserPermission},
            permissions::{NewRessource, UserAccessType, OptionalUserAccessType}, util::AccessType,
        },
        tests::{test::TestRepo, util},
    };

    /// Test for retrieving a user's groups when the user is not a member of any groups.
    ///
    /// This test verifies the behavior of the `get_my_groups` handler when a user requests their groups,
    /// but they are not a member of any groups. The test performs the following steps:
    ///
    /// 1. Creates a test application state and a user repository.
    /// 2. Creates a test user with no group memberships.
    /// 3. Calls the `get_my_groups` handler with an authenticated request for the user's groups.
    /// 4. Asserts that the response status code is "OK" (200).
    /// 5. Validates that the response contains an empty list of groups and a total count of zero.
    /// 6. Clears the test database to remove any created records.
    ///
    /// This test ensures that the handler behaves correctly when a user without group memberships requests their groups.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_my_groups_no_groups() {
        use crate::handlers::user::groups::groups::get_groups;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let resp = test_app
            .call(
                "/api/user/groups/",
                "/api/user/groups",
                get_groups,
                test_app.valid_authorizate(TestRequest::get(), &created_user.id),
            )
            .await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let paging_response: GroupPagingResponse = test::read_body_json(resp).await;
        assert_eq!(paging_response.total_count, 0);
        assert_eq!(paging_response.groups, vec![]);

        test_app.app_state.pgdb.clear_db();
    }

    /// Test for retrieving user's groups when the user is not a member of any groups.
    ///
    /// This test verifies the behavior of the `get_my_groups` 
    /// handler when a user requests their groups, but they are not a member of any groups. 
    /// The test performs the following steps:
    ///
    /// 1. Creates a test application state and a user repository.
    /// 2. Creates a test user with no group memberships.
    /// 3. Calls the `get_my_groups` handler with an authenticated request for 
    /// the user's groups.
    /// 4. Asserts that the response status code is "OK" (200).
    /// 5. Validates that the response contains an empty list of groups and a total count 
    /// of zero.
    /// 6. Clears the test database to remove any created records.
    ///
    /// This test ensures that the handler behaves correctly when a user without 
    /// group memberships requests their groups.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_my_groups_with_groups_but_user_has_no_groups() {
        use crate::handlers::user::groups::groups::get_groups;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        util::create_example_groups(&test_app.group_repo, 2, created_user.id.clone());

        let resp = test_app
            .call(
                "/api/user/groups/",
                "/api/user/groups",
                get_groups,
                test_app.valid_authorizate(TestRequest::get(), &created_user.id),
            )
            .await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let paging_response: GroupPagingResponse = test::read_body_json(resp).await;
        assert_eq!(paging_response.total_count, 0);
        assert_eq!(paging_response.groups, vec![]);

        test_app.app_state.pgdb.clear_db();
    }

    /// Test for retrieving a user's groups when the user has a global permission.
    ///
    /// This test verifies the behavior of the `get_my_groups` handler when a user with global permissions requests their groups. The test performs the following steps:
    ///
    /// 1. Creates a test application state and a user repository.
    /// 2. Creates a test user with a global permission for the "GroupInfo" key.
    /// 3. Creates two groups.
    /// 4. Associates the global permission with the user.
    /// 5. Calls the `get_my_groups` handler with an authenticated request for the user's groups.
    /// 6. Asserts that the response status code is "OK" (200).
    /// 7. Validates that the response contains two groups and a total count of two.
    /// 8. Clears the test database to remove any created records.
    ///
    /// This test ensures that the handler behaves correctly when a user with global permissions for a specific key requests their groups.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_my_groups_with_global_permission() {
        use crate::handlers::user::groups::groups::get_groups;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_groups = util::create_example_groups(&test_app.group_repo, 2, created_user.id.clone());

        let permissions = util::create_ressource(
            &test_app.permission_repo,
            &vec![(NewRessource {
                key_name: &"Group".to_string(),
                key_value: &"group".to_string(),
            }, vec![AccessType::Read])],
        );

        util::create_permissions_for_user(
            &test_app.group_repo,
            &vec![(
                NewUserPermission {
                    user_id: created_user.id,
                    group_id: None,
                    ressource: "group".to_string()
                },
                vec![OptionalUserAccessType {
                    access_type: AccessType::Read,
                    permission: Some(true),
                    set_permission: None,
                    set_set_permission: None,
                }],
            )],
        );

        let resp = test_app
            .call(
                "/api/user/groups/",
                "/api/user/groups",
                get_groups,
                test_app.valid_authorizate(TestRequest::get(), &created_user.id),
            )
            .await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let paging_response: GroupPagingResponse = test::read_body_json(resp).await;
        assert_eq!(paging_response.total_count, 2);
        assert_eq!(
            paging_response.groups,
            vec![
                GroupInfoResponse {
                    id: created_groups[0].id,
                    name: created_groups[0].name.to_string(),
                    parent: None,
                },
                GroupInfoResponse {
                    id: created_groups[1].id,
                    name: created_groups[1].name.to_string(),
                    parent: None,
                }
            ]
        );

        test_app.app_state.pgdb.clear_db();
    }

    /// Test for retrieving a user's groups when the user has specific permissions for a group.
    ///
    /// This test verifies the behavior of the `get_my_groups` handler when a user with specific permissions for a
    /// group requests their groups. The test performs the following steps:
    ///
    /// 1. Creates a test application state and a user repository.
    /// 2. Creates a test user.
    /// 3. Creates two groups.
    /// 4. Associates a specific permission with the user for one of the groups.
    /// 5. Calls the `get_my_groups` handler with an authenticated request for the user's groups.
    /// 6. Asserts that the response status code is "OK" (200).
    /// 7. Validates that the response contains one group and a total count of one.
    /// 8. Clears the test database to remove any created records.
    ///
    /// This test ensures that the handler behaves correctly when a user with specific permissions for a group requests their groups.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_my_groups_with_user_permission() {
        use crate::handlers::user::groups::groups::get_groups;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_groups = util::create_example_groups(&test_app.group_repo, 2, created_user.id.clone());

        let permissions = util::create_ressource(
            &test_app.permission_repo,
            &vec![(NewRessource {
                key_name: &"Group".to_string(),
                key_value: &"group".to_string(),
            }, vec![AccessType::Read])],
        );

        util::create_permissions_for_user(
            &test_app.group_repo,
            &vec![(
                NewUserPermission {
                    user_id: created_user.id,
                    group_id: Some(created_groups[0].id),
                    ressource: "group".to_string()
                },
                vec![OptionalUserAccessType {
                    access_type: AccessType::Read,
                    permission: Some(true),
                    set_permission: None,
                    set_set_permission: None,
                }],
            )],
        );

        let resp = test_app
            .call(
                "/api/user/groups/",
                "/api/user/groups",
                get_groups,
                test_app.valid_authorizate(TestRequest::get(), &created_user.id),
            )
            .await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let paging_response: GroupPagingResponse = test::read_body_json(resp).await;
        assert_eq!(paging_response.total_count, 1);
        assert_eq!(
            paging_response.groups,
            vec![GroupInfoResponse {
                id: created_groups[0].id,
                name: created_groups[0].name.to_string(),
                parent: None,
            }]
        );

        test_app.app_state.pgdb.clear_db();
    }

    /// Test for retrieving a user's specific groups with permissions.
    ///
    /// This test verifies the behavior of the `get_my_groups` handler when a user with specific permissions for certain groups requests their groups. The test performs the following steps:
    ///
    /// 1. Creates a test application state and a user repository.
    /// 2. Creates a test user.
    /// 3. Creates three groups.
    /// 4. Associates specific permissions with the user for two of the groups.
    /// 5. Calls the `get_my_groups` handler with an authenticated request for the user's specific groups.
    /// 6. Asserts that the response status code is "OK" (200).
    /// 7. Validates that the response contains two groups and a total count of two.
    /// 8. Clears the test database to remove any created records.
    ///
    /// This test ensures that the handler behaves correctly when a user with specific permissions for certain groups requests their groups.
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_my_groups_with_specific_groups() {
        use crate::handlers::user::groups::groups::get_groups;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_groups = util::create_example_groups(&test_app.group_repo, 3, created_user.id.clone());

        let permissions = util::create_ressource(
            &test_app.permission_repo,
            &vec![(NewRessource {
                key_name: &"Group".to_string(),
                key_value: &"group".to_string(),
            }, vec![AccessType::Read])],
        );

        util::create_permissions_for_user(
            &test_app.group_repo,
            &vec![(
                NewUserPermission {
                    user_id: created_user.id,
                    group_id: Some(created_groups[0].id),
                    ressource: "group".to_string()
                },
                vec![OptionalUserAccessType {
                    access_type: AccessType::Read,
                    permission: Some(true),
                    set_permission: None,
                    set_set_permission: None,
                }],
            ), (
                NewUserPermission {
                    user_id: created_user.id,
                    group_id: Some(created_groups[1].id),
                    ressource: "group".to_string()
                },
                vec![OptionalUserAccessType {
                    access_type: AccessType::Read,
                    permission: Some(true),
                    set_permission: None,
                    set_set_permission: None,
                }],
            )],
        );

        let resp = test_app
            .call(
                &format!("/api/user/groups/?group_ids[]={}", created_groups[0].id),
                "/api/user/groups",
                get_groups,
                test_app.valid_authorizate(TestRequest::get(), &created_user.id),
            )
            .await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let paging_response: GroupPagingResponse = test::read_body_json(resp).await;
        assert_eq!(paging_response.total_count, 2);
        assert_eq!(
            paging_response.groups,
            vec![GroupInfoResponse {
                id: created_groups[0].id,
                name: created_groups[0].name.to_string(),
                parent: None,
            }]
        );

        test_app.app_state.pgdb.clear_db();
    }

    /// Test for retrieving a user's specific groups that are not available.
    ///
    /// This test verifies the behavior of the `get_my_groups` handler when a user requests specific groups that are not available to them.
    /// The test performs the following steps:
    ///
    /// 1. Creates a test application state and a user repository.
    /// 2. Creates a test user.
    /// 3. Calls the `get_my_groups` handler with an authenticated request for specific groups that don't exist or aren't accessible by the user.
    /// 4. Asserts that the response status code is "OK" (200).
    /// 5. Validates that the response contains no groups and has a total count of zero.
    /// 6. Clears the test database to remove any created records.
    ///
    /// This test ensures that the handler behaves correctly when a user requests specific groups that are not available to them.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_my_groups_specific_groups_not_available() {
        use crate::handlers::user::groups::groups::get_groups;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_groups = util::create_example_groups(&test_app.group_repo, 1, created_user.id.clone());

        let resp = test_app
            .call(
                &format!("/api/user/groups/?group_ids[]={},{}", Uuid::new_v4(), created_groups[0].id),
                "/api/user/groups",
                get_groups,
                test_app.valid_authorizate(TestRequest::get(), &created_user.id),
            )
            .await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let paging_response: GroupPagingResponse = test::read_body_json(resp).await;
        assert_eq!(paging_response.total_count, 0);
        assert_eq!(paging_response.groups, vec![]);

        test_app.app_state.pgdb.clear_db();
    }

    /// Test for retrieving a user's specific groups that may include both unavailable and available groups.
    ///
    /// This test verifies the behavior of the `get_my_groups` handler when a user requests specific groups that may include a mix of unavailable and available groups. The test performs the following steps:
    ///
    /// 1. Creates a test application state and a user repository.
    /// 2. Creates a test user.
    /// 3. Creates a test group, assigns permissions to the user to read the group, and creates a mix of specific group IDs, including an unavailable group and an available group.
    /// 4. Calls the `get_my_groups` handler with an authenticated request for the specified group IDs.
    /// 5. Asserts that the response status code is "OK" (200).
    /// 6. Validates that the response contains the available group and has a total count of one.
    /// 7. Clears the test database to remove any created records.
    ///
    /// This test ensures that the handler behaves correctly when a user requests specific groups that may include both unavailable and available groups, and it should only return the available ones.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_my_groups_specific_groups_not_available_and_available() {
        use crate::handlers::user::groups::groups::get_groups;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_groups = util::create_example_groups(&test_app.group_repo, 1, created_user.id.clone());

        let permissions = util::create_ressource(
            &test_app.permission_repo,
            &vec![(NewRessource {
                key_name: &"Group".to_string(),
                key_value: &"group".to_string(),
            }, vec![AccessType::Read])],
        );

        util::create_permissions_for_user(
            &test_app.group_repo,
            &vec![(
                NewUserPermission {
                    user_id: created_user.id,
                    group_id: Some(created_groups[0].id),
                    ressource: "group".to_string()
                },
                vec![OptionalUserAccessType {
                    access_type: AccessType::Read,
                    permission: Some(true),
                    set_permission: None,
                    set_set_permission: None,
                }],
            )],
        );

        let resp = test_app
            .call(
                &format!("/api/user/groups/?group_ids[]={},{}", Uuid::new_v4(), created_groups[0].id),
                "/api/user/groups",
                get_groups,
                test_app.valid_authorizate(TestRequest::get(), &created_user.id),
            )
            .await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let paging_response: GroupPagingResponse = test::read_body_json(resp).await;
        assert_eq!(paging_response.total_count, 1);
        assert_eq!(
            paging_response.groups,
            vec![GroupInfoResponse {
                id: created_groups[0].id,
                name: created_groups[0].name.to_string(),
                parent: None,
            }]
        );

        test_app.app_state.pgdb.clear_db();
    }

    /// Test for retrieving a user's groups with pagination.
    ///
    /// This test verifies the behavior of the `get_my_groups` handler when a user requests their groups with pagination. The test performs the following steps:
    ///
    /// 1. Creates a test application state and a user repository.
    /// 2. Creates a test user.
    /// 3. Creates multiple test groups and assigns permissions to the user to read the groups.
    /// 4. Calls the `get_my_groups` handler with an authenticated request for the second page of groups with a limit of one group per page.
    /// 5. Asserts that the response status code is "OK" (200).
    /// 6. Validates that the response contains the expected group for the second page and has a total count of all available groups.
    /// 7. Clears the test database to remove any created records.
    ///
    /// This test ensures that the handler supports pagination correctly, returning the expected results for a specified page and limit.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_my_groups_paging() {
        use crate::handlers::user::groups::groups::get_groups;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_groups = util::create_example_groups(&test_app.group_repo, 3, created_user.id.clone());

        let permissions = util::create_ressource(
            &test_app.permission_repo,
            &vec![(NewRessource {
                key_name: &"Group".to_string(),
                key_value: &"group".to_string(),
            }, vec![AccessType::Read])],
        );

        util::create_permissions_for_user(
            &test_app.group_repo,
            &vec![(
                NewUserPermission {
                    user_id: created_user.id,
                    group_id: None,
                    ressource: "group".to_string()
                },
                vec![OptionalUserAccessType {
                    access_type: AccessType::Read,
                    permission: Some(true),
                    set_permission: None,
                    set_set_permission: None,
                }],
            )],
        );

        let resp = test_app
            .call(
                "/api/user/groups/?page=2&limit=1&order=DESC",
                "/api/user/groups",
                get_groups,
                test_app.valid_authorizate(TestRequest::get(), &created_user.id),
            )
            .await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let paging_response: GroupPagingResponse = test::read_body_json(resp).await;
        assert_eq!(paging_response.total_count, 3);
        assert_eq!(
            paging_response.groups,
            vec![GroupInfoResponse {
                id: created_groups[2].id,
                name: created_groups[2].name.to_string(),
                parent: None,
            }]
        );

        test_app.app_state.pgdb.clear_db();
    }

    /// Test for retrieving a user's groups with ascending pagination.
    ///
    /// This test verifies the behavior of the `get_my_groups` handler when a user requests their groups with ascending pagination. The test performs the following steps:
    ///
    /// 1. Creates a test application state and a user repository.
    /// 2. Creates a test user.
    /// 3. Creates multiple test groups and assigns permissions to the user to read the groups.
    /// 4. Calls the `get_my_groups` handler with an authenticated request for the second page of groups with a limit of one group per page and ascending order.
    /// 5. Asserts that the response status code is "OK" (200).
    /// 6. Validates that the response contains the expected group for the second page in ascending order and has a total count of all available groups.
    /// 7. Clears the test database to remove any created records.
    ///
    /// This test ensures that the handler supports ascending pagination correctly, returning the expected results in ascending order for a specified page and limit.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_my_groups_paging_asc() {
        use crate::handlers::user::groups::groups::get_groups;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_groups = util::create_example_groups(&test_app.group_repo, 3, created_user.id.clone());

        let permissions = util::create_ressource(
            &test_app.permission_repo,
            &vec![(NewRessource {
                key_name: &"Group".to_string(),
                key_value: &"group".to_string(),
            }, vec![AccessType::Read])],
        );

        util::create_permissions_for_user(
            &test_app.group_repo,
            &vec![(
                NewUserPermission {
                    user_id: created_user.id,
                    group_id: None,
                    ressource: "group".to_string()
                },
                vec![OptionalUserAccessType {
                    access_type: AccessType::Read,
                    permission: Some(true),
                    set_permission: None,
                    set_set_permission: None,
                }],
            )],
        );
        let resp = test_app
            .call(
                "/api/user/groups/?page=2&limit=1&order=ASC",
                "/api/user/groups",
                get_groups,
                test_app.valid_authorizate(TestRequest::get(), &created_user.id),
            )
            .await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let paging_response: GroupPagingResponse = test::read_body_json(resp).await;
        assert_eq!(paging_response.total_count, 3);
        assert_eq!(
            paging_response.groups,
            vec![GroupInfoResponse {
                id: created_groups[0].id,
                name: created_groups[0].name.to_string(),
                parent: None,
            }]
        );

        test_app.app_state.pgdb.clear_db();
    }
}
