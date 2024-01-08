//docu written with chat-gpt
#[cfg(test)]
mod fetch_members_tests {
    use actix_web::{test::{TestRequest, self}, http};
    use serde_json::Value;
    use uuid::Uuid;

    use crate::{tests::{util, test::TestRepo}, models::{permissions::{NewRessource, OptionalUserAccessType}, groups::{NewUserPermission, CreateGroupResponse, GroupInfoResponse, GroupMetaDataResponse}, util::{AccessType, PagingSchema, OrderDir}, task::TaskPagingResponse, roles::{NewRole, UpdateRolePermission, NewRolePermission, NewRoleAccessType, UpdateRoleAccesType}, task_package::TaskPackagesResponse, members::MemberListWithCountResponse, auth::RegisterUserSchema}};

    static SCOPE: &'static str = "/api/groups/{group_id}/members"; 
    
    fn get_path(group_id: &Uuid, query: &str) -> String {
        format!("/api/groups/{}/members/{}", group_id, query)
    }

    /// # Test: `test_fetch_members`
    ///
    /// Validates the behavior of the `get_group_members` handler in Actix-Web when fetching members of a group. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a test user (`created_user`) and an example group (`created_groups`) associated with `created_user`.
    ///    - Creates a permission for the user to read group members.
    ///    - Adds a member (`member`) to the group.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `get_group_members` using the test application instance with a valid authorization token for `created_user`.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "OK" (200), indicating a successful request.
    ///    - Verifies that the response contains the correct total count of members (1).
    ///    - Verifies that the response contains the correct member details for the added member.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_fetch_members() {
        use crate::handlers::groups::group_id::members::members::get_group_members;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_groups = util::create_example_groups(&test_app.group_repo, 1, created_user.id);

        util::create_permissions(&test_app.permission_repo, &test_app.group_repo, "member", &created_user.id, &AccessType::Read);

        let member = util::add_member_to_group(&test_app.group_repo, &created_user.id, &created_groups[0].id);

        let resp = test_app
        .call(
            &get_path(&created_groups[0].id, ""),
            SCOPE,
            get_group_members,
            test_app.valid_authorizate(TestRequest::get(), &created_user.id),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let response: MemberListWithCountResponse = test::read_body_json(resp).await;

        assert_eq!(response.total_count, 1);
        assert_eq!(response.members.len(), 1);
        assert_eq!(response.members[0].id, member.member_id);
    }

    /// # Test: `test_fetch_members_member_id`
    ///
    /// Validates the behavior of the `get_group_members` handler in Actix-Web when fetching members of a group based on specific member IDs. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates two test users (`created_user` and `other_user`) and an example group (`created_groups`) associated with `created_user`.
    ///    - Creates a permission for `created_user` to read group members.
    ///    - Adds two members (`member` and `member_other`) to the group, each associated with a different user.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `get_group_members` using the test application instance with a valid authorization token for `created_user` and a query parameter specifying a single member ID (`member.member_id`).
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "OK" (200), indicating a successful request.
    ///    - Verifies that the response contains the correct total count of members (2).
    ///    - Verifies that the response contains only the member details for the specified member ID (`member.member_id`).
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_fetch_members_member_id() {
        use crate::handlers::groups::group_id::members::members::get_group_members;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);
        let other_user = util::create_other_test_user(&test_app.user_repo);

        let created_groups = util::create_example_groups(&test_app.group_repo, 1, created_user.id);

        util::create_permissions(&test_app.permission_repo, &test_app.group_repo, "member", &created_user.id, &AccessType::Read);

        let member = util::add_member_to_group(&test_app.group_repo, &created_user.id, &created_groups[0].id);
        let member_other = util::add_member_to_group(&test_app.group_repo, &other_user.id, &created_groups[0].id);

        let resp = test_app
        .call(
            &get_path(&created_groups[0].id, &format!("?member_ids[]={}", member.member_id)),
            SCOPE,
            get_group_members,
            test_app.valid_authorizate(TestRequest::get(), &created_user.id),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let response: MemberListWithCountResponse = test::read_body_json(resp).await;

        assert_eq!(response.total_count, 2);
        assert_eq!(response.members.len(), 1);
        assert_eq!(response.members[0].id, member.member_id);
    }

    /// # Test: `test_fetch_members_paging`
    ///
    /// Validates the behavior of the `get_group_members` handler in Actix-Web when fetching members of a group with pagination. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates three test users (`created_user`, `other_user`, and `third_user`) and an example group (`created_groups`) associated with `created_user`.
    ///    - Creates a permission for `created_user` to read group members.
    ///    - Adds three members to the group (`member`, `member_other`, and `member_third`), each associated with a different user.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `get_group_members` using the test application instance with a valid authorization token for `created_user` and query parameters specifying a limit of 2 members per page and requesting page 1.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "OK" (200), indicating a successful request.
    ///    - Verifies that the response contains the correct total count of members (3).
    ///    - Verifies that the response contains only the member details for the members on page 1 (`member_third`).
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_fetch_members_paging() {
        use crate::handlers::groups::group_id::members::members::get_group_members;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);
        let other_user = util::create_other_test_user(&test_app.user_repo);
        let third_user = util::create_test_user(
            &&RegisterUserSchema {
                name: "T".to_string(),
                email: "T@T.de".to_string(),
                password: "123".to_string(),
            },
            &test_app.user_repo.clone(),
        );

        let created_groups = util::create_example_groups(&test_app.group_repo, 1, created_user.id);

        util::create_permissions(&test_app.permission_repo, &test_app.group_repo, "member", &created_user.id, &AccessType::Read);

        let member = util::add_member_to_group(&test_app.group_repo, &created_user.id, &created_groups[0].id);
        let member_other = util::add_member_to_group(&test_app.group_repo, &other_user.id, &created_groups[0].id);
        let member_third = util::add_member_to_group(&test_app.group_repo, &third_user.id, &created_groups[0].id);

        let resp = test_app
        .call(
            &get_path(&created_groups[0].id, &format!("?limit=2&page=1")),
            SCOPE,
            get_group_members,
            test_app.valid_authorizate(TestRequest::get(), &created_user.id),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let response: MemberListWithCountResponse = test::read_body_json(resp).await;

        assert_eq!(response.total_count, 3);
        assert_eq!(response.members.len(), 1);
        assert_eq!(response.members[0].id, member_third.member_id);
    }

    /// # Test: `test_fetch_members_paging_member_id`
    ///
    /// Validates the behavior of the `get_group_members` handler in Actix-Web when fetching members of a group with pagination and specific member IDs. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates three test users (`created_user`, `other_user`, and `third_user`) and an example group (`created_groups`) associated with `created_user`.
    ///    - Creates a permission for `created_user` to read group members.
    ///    - Adds three members to the group (`member`, `member_other`, and `member_third`), each associated with a different user.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `get_group_members` using the test application instance with a valid authorization token for `created_user` and query parameters specifying a limit of 1 member per page, requesting page 1, and filtering by specific member IDs (`member.member_id` and `member_other.member_id`).
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "OK" (200), indicating a successful request.
    ///    - Verifies that the response contains the correct total count of members (3).
    ///    - Verifies that the response contains only the member details for the members on the requested page (page 1) and matching the specified member IDs (`member_other`).
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_fetch_members_paging_member_id() {
        use crate::handlers::groups::group_id::members::members::get_group_members;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);
        let other_user = util::create_other_test_user(&test_app.user_repo);
        let third_user = util::create_test_user(
            &&RegisterUserSchema {
                name: "T".to_string(),
                email: "T@T.de".to_string(),
                password: "123".to_string(),
            },
            &test_app.user_repo.clone(),
        );

        let created_groups = util::create_example_groups(&test_app.group_repo, 1, created_user.id);

        util::create_permissions(&test_app.permission_repo, &test_app.group_repo, "member", &created_user.id, &AccessType::Read);

        let member = util::add_member_to_group(&test_app.group_repo, &created_user.id, &created_groups[0].id);
        let member_other = util::add_member_to_group(&test_app.group_repo, &other_user.id, &created_groups[0].id);
        let member_third = util::add_member_to_group(&test_app.group_repo, &third_user.id, &created_groups[0].id);

        let resp = test_app
        .call(
            &get_path(&created_groups[0].id, &format!("?limit=1&page=1&member_ids[]={},{}", member.member_id, member_other.member_id)),
            SCOPE,
            get_group_members,
            test_app.valid_authorizate(TestRequest::get(), &created_user.id),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let response: MemberListWithCountResponse = test::read_body_json(resp).await;

        assert_eq!(response.total_count, 3);
        assert_eq!(response.members.len(), 1);
        assert_eq!(response.members[0].id, member_other.member_id);
    }

    /// # Test: `test_fetch_members_no_auth`
    ///
    /// Validates the behavior of the `get_group_members` handler in Actix-Web when attempting to fetch members of a group without proper authorization. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a test user (`created_user`) and an example group (`created_groups`) associated with `created_user`.
    ///    - Creates a permission for `created_user` to read group members.
    ///    - Adds a member to the group (`member`) associated with `created_user`.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `get_group_members` using the test application instance with an invalid (unauthorized) authorization token for `created_user`.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "UNAUTHORIZED" (401), indicating a lack of proper authorization for the request.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_fetch_members_no_auth() {
        use crate::handlers::groups::group_id::members::members::get_group_members;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_groups = util::create_example_groups(&test_app.group_repo, 1, created_user.id);

        util::create_permissions(&test_app.permission_repo, &test_app.group_repo, "member", &created_user.id, &AccessType::Read);

        let member = util::add_member_to_group(&test_app.group_repo, &created_user.id, &created_groups[0].id);

        let resp = test_app
        .call(
            &get_path(&created_groups[0].id, ""),
            SCOPE,
            get_group_members,
            test_app.invalid_authorizate(TestRequest::get(), &created_user.id),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);
    }

    /// # Test: `test_fetch_no_permission`
    ///
    /// Validates the behavior of the `get_group_members` handler in Actix-Web when attempting to fetch members of a group without the required permissions. The test covers the following steps:
    ///
    /// 1. Setup:
    ///    - Creates a test environment with a `TestRepo` instance.
    ///    - Generates a test user (`created_user`) and an example group (`created_groups`) associated with `created_user`.
    ///    - Adds a member to the group (`member`) associated with `created_user`.
    ///
    /// 2. Execution:
    ///    - Makes an API call to `get_group_members` using the test application instance with a valid authorization token for `created_user`, but without the necessary permissions to read group members.
    ///
    /// 3. Assertions:
    ///    - Verifies that the response status is "FORBIDDEN" (403), indicating a lack of required permissions for the request.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_fetch_no_permission() {
        use crate::handlers::groups::group_id::members::members::get_group_members;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let created_groups = util::create_example_groups(&test_app.group_repo, 1, created_user.id);

        let member = util::add_member_to_group(&test_app.group_repo, &created_user.id, &created_groups[0].id);

        let resp = test_app
        .call(
            &get_path(&created_groups[0].id, ""),
            SCOPE,
            get_group_members,
            test_app.valid_authorizate(TestRequest::get(), &created_user.id),
        )
        .await;
        assert_eq!(resp.status(), http::StatusCode::FORBIDDEN);
    }
}