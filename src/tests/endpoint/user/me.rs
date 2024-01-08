#[cfg(test)]
mod me_tests {
    use actix_web::{test::{TestRequest, self}, http};

    use crate::{models::{auth::RegisterUserSchema, users::UserResponse}, tests::{util, test::TestRepo}};

    static SCOPE: &'static str = "/api/user"; 
    
    fn get_path() -> String {
        "/api/user/".to_string()
    }

    /// Test for successfully retrieving the authenticated user's profile.
    ///
    /// This test verifies the behavior of the `me` handler when a user requests their own profile. The test performs the following steps:
    ///
    /// 1. Creates a test application state and a user repository.
    /// 2. Creates a test user.
    /// 3. Calls the `me` handler with an authenticated request to retrieve the user's profile.
    /// 4. Asserts that the response status code is "OK" (200).
    /// 5. Validates that the response contains the expected user profile data, including the user's ID, name, and email.
    ///
    /// This test ensures that the `me` handler correctly retrieves the user's profile and returns the expected user data.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_me_successful() {
        use crate::handlers::user::user::me;

        let test_app = TestRepo::new().await;

        let created_user = util::create_test_user(
            &RegisterUserSchema {
                email: "test@test.de".to_string(),
                name: "Test".to_string(),
                password: "123".to_string(),
            },
            &test_app.user_repo,
        );

        let resp = test_app
            .call(
                &get_path(),
                SCOPE,
                me,
                test_app.valid_authorizate(TestRequest::get(), &created_user.id),
            )
            .await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        let user_response: UserResponse = test::read_body_json(resp).await;
        util::assert_user_response(&user_response, &created_user);
        test_app.app_state.pgdb.clear_db();
    }

    /// Test for an unsuccessful attempt to retrieve the authenticated user's profile without proper authorization.
    ///
    /// This test verifies the behavior of the `me` handler when a user attempts to access their profile without proper authorization. The test performs the following steps:
    ///
    /// 1. Creates a test application state and a user repository.
    /// 2. Creates a test user.
    /// 3. Calls the `me` handler with an unauthorized request to retrieve the user's profile.
    /// 4. Asserts that the response status code is "UNAUTHORIZED" (401).
    ///
    /// This test ensures that the `me` handler correctly handles unauthorized requests and returns the expected "UNAUTHORIZED" status code.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_me_unsuccessful() {
        use crate::handlers::user::user::me;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let resp = test_app
            .call(
                &get_path(),
                SCOPE,
                me,
                test_app.invalid_authorizate(TestRequest::get(), &created_user.id),
            )
            .await;
        assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);
        test_app.app_state.pgdb.clear_db();
    }
}