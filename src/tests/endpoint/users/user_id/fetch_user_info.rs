//Doku written with chat-gpt
#[cfg(test)]
mod fetch_user_info_tests {
    use actix_web::{test::{self,TestRequest}, http};
    use uuid::Uuid;

    use crate::{tests::{test::TestRepo, util}, models::{auth::RegisterUserSchema, users::UserResponse}};

    static SCOPE: &'static str = "/api/users/{user_id}"; 
    
    fn get_path(user_id: &Uuid, query: &str) -> String {
        format!("/api/users/{}/info{}", user_id, query)
    }

    /// Test for fetching user information.
    ///
    /// This test verifies the behavior of the `fetch_user_info` handler when a user attempts to retrieve information about another user. The test performs the following steps:
    ///
    /// 1. Creates a test application state and a user repository.
    /// 2. Creates two test users: `created_me` and `created_other`.
    /// 3. Calls the `fetch_user_info` handler with an authorized request using `created_me`'s credentials to retrieve information about `created_other`.
    /// 4. Asserts that the response status code is "OK" (200).
    /// 5. Compares the response data to the `created_other` user's information to ensure that the correct user's data is returned.
    ///
    /// This test ensures that the `fetch_user_info` handler correctly retrieves and returns user information and handles the request with proper authorization.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_fetch_user_info() {
        use crate::handlers::users::user_id::user_id::fetch_user_info;

        let test_app = TestRepo::new().await;

        let created_me = util::create_standard_test_user(&test_app.user_repo);

        let created_other = util::create_test_user(
            &RegisterUserSchema {
                email: "test2@test.de".to_string(),
                name: "Test2".to_string(),
                password: "1234".to_string(),
            },
            &test_app.user_repo,
        );

        let resp = test_app
            .call(
                &get_path(&created_other.id, ""),
                SCOPE,
                fetch_user_info,
                test_app.valid_authorizate(TestRequest::get(), &created_me.id),
            )
            .await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let user_response: UserResponse = test::read_body_json(resp).await;
        util::assert_user_response(&user_response, &created_other);
        
        test_app.app_state.pgdb.clear_db();
    }

    /// Test for fetching user information with an invalid token.
    ///
    /// This test verifies the behavior of the `fetch_user_info` handler when a user attempts to retrieve information about another user with an invalid token. The test performs the following steps:
    ///
    /// 1. Creates a test application state and a user repository.
    /// 2. Creates two test users: `created_me` and `created_other`.
    /// 3. Calls the `fetch_user_info` handler with an unauthorized request using an invalid token.
    /// 4. Asserts that the response status code is "UNAUTHORIZED" (401).
    ///
    /// This test ensures that the `fetch_user_info` handler correctly handles unauthorized requests with invalid tokens.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_fetch_user_info_invalid_token() {
        use crate::handlers::users::user_id::user_id::fetch_user_info;

        let test_app = TestRepo::new().await;

        let created_me = util::create_standard_test_user(&test_app.user_repo);

        let created_other = util::create_test_user(
            &RegisterUserSchema {
                email: "test2@test.de".to_string(),
                name: "Test2".to_string(),
                password: "1234".to_string(),
            },
            &test_app.user_repo,
        );

        let resp = test_app
            .call(
                &get_path(&created_other.id, ""),
                SCOPE,
                fetch_user_info,
                test_app.invalid_authorizate(TestRequest::get(), &created_me.id),
            )
            .await;
        assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);
        test_app.app_state.pgdb.clear_db();
    }

    /// Test for fetching information of a non-existing user.
    ///
    /// This test verifies the behavior of the `fetch_user_info` handler when a user attempts to retrieve information about a non-existing user. The test performs the following steps:
    ///
    /// 1. Creates a test application state and a user repository.
    /// 2. Creates a test user named `created_me`.
    /// 3. Generates a UUID `not_existing_id` that doesn't correspond to any existing user.
    /// 4. Calls the `fetch_user_info` handler with a request to retrieve information about the non-existing user using a valid token.
    /// 5. Asserts that the response status code is "NOT_FOUND" (404).
    ///
    /// This test ensures that the `fetch_user_info` handler correctly handles requests to retrieve information about non-existing users.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_fetch_user_info_not_existing_user() {
        use crate::handlers::users::user_id::user_id::fetch_user_info;

        let test_app = TestRepo::new().await;

        let created_me = util::create_standard_test_user(&test_app.user_repo);

        let not_existing_id = Uuid::new_v4();

        assert_ne!(not_existing_id, created_me.id);

        let resp = test_app
            .call(
                &get_path(&not_existing_id, ""),
                SCOPE,
                fetch_user_info,
                test_app.valid_authorizate(TestRequest::get(), &created_me.id),
            )
            .await;
        assert_eq!(resp.status(), http::StatusCode::NOT_FOUND);
        test_app.app_state.pgdb.clear_db();
    }
}