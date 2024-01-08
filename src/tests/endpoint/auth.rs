//Doku written with chat-gpt
#[cfg(test)]
mod auth_tests {
    use actix_web::{
        cookie::{time::Duration, Cookie},
        http,
        test::{self, TestRequest},
    };
    use uuid::Uuid;

    use crate::tests::test::TestRepo;

    use crate::{
        models::{
            auth::{LoginResponse, LoginUserSchema, RegisterUserSchema},
            users::UserResponse,
            util::ErrorSchema,
        },
        tests::util::{self},
    };

    /// Test for ensuring that a successful login generates a valid token.
    ///
    /// This test initializes the application state, creates a test user, sends a login request,
    /// validates the response, and ensures that the generated token is valid.
    ///
    /// Steps:
    /// 1. Initialize the application state and create a user repository.
    /// 2. Create a test user with a known email and password.
    /// 3. Send a login request with the user's email and password.
    /// 4. Verify that the response status code is OK (200).
    /// 5. Extract the token from the response's "Set-Cookie" header and validate its claims.
    /// 6. Parse the response body to extract the token and validate its claims.
    /// 7. Delete the test user.
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_successful_login_generates_valid_token() {
        let user = &RegisterUserSchema {
            name: "Test".to_string(),
            email: "Test@test.de".to_string(),
            password: "123".to_string(),
        };

        let login = &LoginUserSchema {
            email: user.email.to_string(),
            password: user.password.to_string(),
        };

        test_successful_login_generates_valid_token_body(user, login).await;
    }

    /// Test for verifying that a successful login generates a valid token with case-insensitive email comparison.
    ///
    /// This test demonstrates the behavior of the login process when the user attempts to log in using an email with a
    /// different case from the registered user's email. The test uses two schemas: `user` represents the user already
    /// registered with a lowercase email, and `login` represents the login attempt with a different case email.
    ///
    /// Steps:
    /// 1. Initialize the application state and create a user repository.
    /// 2. Create a test user with the same email as `user`.
    /// 3. Create an Actix web application for testing and configure it with the application state and user repository.
    /// 4. Prepare the login data for `login`.
    /// 5. Send a login request with the prepared data to the login endpoint.
    /// 6. Verify that the response status code is OK (200).
    /// 7. Extract the token from the response's Set-Cookie header and validate its claims.
    /// 8. Parse the response body to extract the login response.
    /// 9. Validate the token claims within the login response.
    /// 10. Delete the registered user.
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_successful_login_generates_valid_token_case_insensitive() {
        let user = &RegisterUserSchema {
            name: "Test".to_string(),
            email: "Test@test.de".to_string(),
            password: "123".to_string(),
        };

        let login = &LoginUserSchema {
            email: "tEst@tEst.de".to_string(),
            password: user.password.to_string(),
        };

        test_successful_login_generates_valid_token_body(user, login).await;
    }

    #[cfg(test)]
    async fn test_successful_login_generates_valid_token_body(
        user: &RegisterUserSchema,
        login_user: &LoginUserSchema,
    ) {
        use crate::handlers::auth::auth::login;

        let test_app = TestRepo::new().await;
        
        let created_user = util::create_test_user(user, &test_app.user_repo.clone());

        let request_value = serde_json::json!({
            "email": login_user.email,
            "password": login_user.password,
        });

        let resp = test_app.call(
            "/login", 
            "/api/auth",
            login, 
            TestRequest::post().set_json(request_value)
        ).await;
        assert_eq!(resp.status(), http::StatusCode::OK);

        {
            let set_cookie_header = resp
                .headers()
                .get(actix_web::http::header::SET_COOKIE)
                .unwrap();
            let set_cookie_value = set_cookie_header.to_str().unwrap();

            let cookie = Cookie::parse(set_cookie_value).unwrap();
            let token_from_cookie = cookie.value();

            util::validate_token_claims(
                util::decode_token_claims(token_from_cookie, &test_app.app_state),
                created_user.id.to_string(),
            );
        }

        let resp_body: LoginResponse = test::read_body_json(resp).await;
        util::validate_token_claims(
            util::decode_token_claims(&resp_body.token, &test_app.app_state),
            created_user.id.to_string(),
        );
    }

    /// Test for verifying that an incorrect password during login generates the correct response.
    ///
    /// This test initializes the application state, creates a test user, sends a login request
    /// with an incorrect password, and ensures that the response is as expected.
    ///
    /// Steps:
    /// 1. Initialize the application state and create a user repository.
    /// 2. Create a test user with known email and password.
    /// 3. Send a login request with the user's email and an incorrect password.
    /// 4. Verify that the response status code is UNAUTHORIZED (401).
    /// 5. Parse the response body to extract the error message and validate it.
    /// 6. Delete the test user.
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_wrong_password_login_generates_correct_response() {
        use crate::handlers::auth::auth::login;

        let test_app = TestRepo::new().await;

        let created_user = util::create_test_user(
            &RegisterUserSchema {
                name: "Test".to_string(),
                email: "Test@test.de".to_string(),
                password: "123".to_string(),
            },
            &test_app.user_repo.clone(),
        );

        let request_body = serde_json::json!({
            "email": created_user.email.to_string(),
            "password": "12",
        });

        let resp = test_app.call(
            "/login", 
            "/api/auth",
            login, 
            TestRequest::post().set_json(&request_body)
        ).await;
        assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);

        let resp_body: ErrorSchema = test::read_body_json(resp).await;
        assert_eq!(resp_body.message, "Invalid email or password");

        test_app.app_state.pgdb.clear_db();
    }

    /// Test for verifying that an incorrect email during login generates the correct response.
    ///
    /// This test initializes the application state, creates a test user, sends a login request
    /// with an incorrect email, and ensures that the response is as expected.
    ///
    /// Steps:
    /// 1. Initialize the application state and create a user repository.
    /// 2. Create a test user with a known email and password.
    /// 3. Send a login request with an incorrect email and the user's correct password.
    /// 4. Verify that the response status code is UNAUTHORIZED (401).
    /// 5. Parse the response body to extract the error message and validate it.
    /// 6. Delete the test user.
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_wrong_email_login_generates_correct_response() {
        use crate::handlers::auth::auth::login;

        let test_app = TestRepo::new().await;

        util::create_test_user(
            &RegisterUserSchema {
                name: "Test".to_string(),
                email: "Test@test.de".to_string(),
                password: "123".to_string(),
            },
            &test_app.user_repo.clone(),
        );

        let request_body = serde_json::json!({
            "email": "Test",
            "password": "123",
        });

        let resp = test_app.call(
            "/login", 
            "/api/auth",
            login, 
            TestRequest::post().set_json(&request_body),
        ).await;
        assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);

        let resp_body: ErrorSchema = test::read_body_json(resp).await;
        assert_eq!(resp_body.message, "Invalid email or password");

        test_app.app_state.pgdb.clear_db();
    }

    /// Test for verifying a successful user registration.
    ///
    /// This test initializes the application state, sends a registration request with valid user data,
    /// and ensures that the response is as expected.
    ///
    /// Steps:
    /// 1. Initialize the application state and create a user repository.
    /// 2. Create an Actix web application for testing and configure it with the application state
    ///    and user repository.
    /// 3. Prepare user registration data, including the user's name, email, and password.
    /// 4. Send a registration request with the prepared data to the registration endpoint.
    /// 5. Verify that the response status code is CREATED (201), indicating a successful registration.
    /// 6. Parse the response body to extract the registered user's information.
    /// 7. Perform assertions to validate the registered user's data against the provided registration data.
    /// 8. Verify that there is one user registered in the database.
    /// 9. Delete the registered user.
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_successful_register() {
        use crate::handlers::auth::auth::register;

        let test_app = TestRepo::new().await;

        let password = "123";

        let request_body = serde_json::json!({
            "name": "Test",
            "email": "test@test.de",
            "password": password,
        });

        let resp = test_app.call(
            "/register",
            "/api/auth",
            register,
            TestRequest::post().set_json(&request_body),
        ).await;
        assert_eq!(resp.status(), http::StatusCode::CREATED);

        let resp_body: UserResponse = test::read_body_json(resp).await;
        util::assert_user(&test_app.user_repo, &resp_body, password);

        assert_eq!(test_app.user_repo.fetch_amount_user().unwrap(), 1);

        test_app.app_state.pgdb.clear_db();
    }

    /// Test for verifying that registration with an already used email returns the correct response.
    ///
    /// This test initializes the application state, creates a user repository, and registers a user with a
    /// specific email address. Then, it attempts to register another user with the same email address and
    /// verifies that the response is as expected.
    ///
    /// Steps:
    /// 1. Initialize the application state and create a user repository.
    /// 2. Create a test user with a known email address.
    /// 3. Create an Actix web application for testing and configure it with the application state
    ///    and user repository.
    /// 4. Prepare registration data with the same email address as the previously created user.
    /// 5. Send a registration request with the prepared data to the registration endpoint.
    /// 6. Verify that the response status code is CONFLICT (409), indicating a registration conflict.
    /// 7. Parse the response body to extract the error message.
    /// 8. Assert that the error message matches the expected message: "User with that email already exists".
    /// 9. Perform additional assertions:
    ///    - Validate that the previously created user's data remains unchanged.
    ///    - Ensure there is only one user in the database.
    /// 10. Delete the registered user.
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_register_email_already_used() {
        let db_user = RegisterUserSchema {
            name: "Test".to_string(),
            email: "Test@test.de".to_string(),
            password: "452".to_string(),
        };
        let register_user = RegisterUserSchema {
            name: "Test".to_string(),
            email: "Test@test.de".to_string(),
            password: "123".to_string(),
        };

        test_register_email_already_used_body(&db_user, &register_user).await;
    }

    /// Test for verifying that registration with an already used email (case-insensitive) returns the correct response.
    ///
    /// This test demonstrates the behavior of the registration process when a new user attempts to register with an email
    /// that differs only in case from an existing user's email. The test uses two registration schemas: `db_user` and
    /// `register_user`, where `db_user` represents the user already in the database, and `register_user` is the new user
    /// with an email that only differs in case from the existing user's email.
    ///
    /// Steps:
    /// 1. Initialize the application state and create a user repository.
    /// 2. Create a test user with the same email as `db_user`.
    /// 3. Create an Actix web application for testing and configure it with the application state and user repository.
    /// 4. Prepare registration data for `register_user`.
    /// 5. Send a registration request with the prepared data to the registration endpoint.
    /// 6. Verify that the response status code is CONFLICT (409), indicating a registration conflict.
    /// 7. Parse the response body to extract the error message.
    /// 8. Assert that the error message matches the expected message: "User with that email already exists".
    /// 9. Perform additional assertions:
    ///    - Validate that the previously created user's data remains unchanged.
    ///    - Ensure there is only one user in the database.
    /// 10. Delete the registered user.
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_register_email_already_used_email_case_insensitive() {
        let db_user = RegisterUserSchema {
            name: "Test".to_string(),
            email: "Test@test.de".to_string(),
            password: "452".to_string(),
        };
        let register_user = RegisterUserSchema {
            name: "Test".to_string(),
            email: "tesT@tEst.de".to_string(),
            password: "123".to_string(),
        };

        test_register_email_already_used_body(&db_user, &register_user).await;
    }

    #[cfg(test)]
    async fn test_register_email_already_used_body(
        db_user: &RegisterUserSchema,
        register_user: &RegisterUserSchema,
    ) {
        use crate::handlers::auth::auth::register;

        let test_app = TestRepo::new().await;

        let created_user = util::create_test_user(db_user, &test_app.user_repo.clone());

        let request_body = serde_json::json!({
            "name": register_user.name,
            "email": register_user.email,
            "password": register_user.password,
        });

        let resp = test_app.call(
            "/register",
            "/api/auth",
            register,
            TestRequest::post().set_json(&request_body),
        ).await;
        assert_eq!(resp.status(), http::StatusCode::CONFLICT);

        let resp_body: ErrorSchema = test::read_body_json(resp).await;
        assert_eq!(resp_body.message, "User with that email already exists");

        util::assert_user(
            &test_app.user_repo,
            &UserResponse {
                email: created_user.email,
                name: created_user.name,
                id: created_user.id,
            },
            &db_user.password,
        );

        assert_eq!(test_app.user_repo.fetch_amount_user().unwrap(), 1);

        test_app.app_state.pgdb.clear_db();
    }

    /// Test for validating that a successful token validation returns a "NO_CONTENT" (204) status code.
    ///
    /// This test verifies that the token validation endpoint works correctly when a valid access token is provided.
    /// It performs the following steps:
    ///
    /// 1. Initializes the application state and creates a user repository.
    /// 2. Creates a test user and generates a valid access token for this user.
    /// 3. Initializes an Actix web application for testing and configures it with the application state.
    /// 4. Sends a GET request to the validation endpoint (/validate) with the valid access token in the "Authorization" header.
    /// 5. Verifies that the response status code is "NO_CONTENT" (204), indicating successful validation.
    /// 6. Deletes the test user.
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_successful_validate_token() {
        use crate::handlers::auth::auth::validate;

        let test_app = TestRepo::new().await;

        let created_user = util::create_test_user(
            &RegisterUserSchema {
                name: "Test".to_string(),
                email: "Test@test.de".to_string(),
                password: "123".to_string(),
            },
            &test_app.user_repo.clone(),
        );

        let resp = test_app.call(
            "/validate",
            "/api/auth",
            validate,
            test_app.valid_authorizate(TestRequest::get(), &created_user.id)
        ).await;
        assert_eq!(resp.status(), http::StatusCode::NO_CONTENT);

        test_app.app_state.pgdb.clear_db();
    }

    /// Test for validating that a valid access token in a cookie returns a "NO_CONTENT" (204) status code.
    ///
    /// This test checks the behavior of the token validation endpoint when a valid access token is provided in a cookie. It performs the following steps:
    ///
    /// 1. Initializes the application state.
    /// 2. Creates a test user and an associated user repository.
    /// 3. Initializes an Actix web application for testing and configures it with the application state.
    /// 4. Generates a valid access token using a known user's ID.
    /// 5. Constructs a cookie containing the valid access token and sends a GET request to the validation endpoint (/validate) with the cookie.
    /// 6. Verifies that the response status code is "NO_CONTENT" (204), indicating that the token is valid.
    /// 7. Deletes the test user.
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_successful_validate_token_cookie() {
        use crate::handlers::auth::auth::validate;

        let test_app = TestRepo::new().await;

        let created_user = util::create_test_user(
            &RegisterUserSchema {
                name: "Test".to_string(),
                email: "Test@test.de".to_string(),
                password: "123".to_string(),
            },
            &test_app.user_repo.clone(),
        );

        let valid_access_token = util::create_valid_jwt_token(&created_user.id, &test_app.app_state);

        let cookie = Cookie::build("token", valid_access_token)
            .path("/")
            .max_age(Duration::new(60 * 60, 0))
            .http_only(true)
            .finish();

        let resp = test_app.call(
            "/validate",
            "/api/auth",
            validate,
            TestRequest::get().cookie(cookie),
        ).await;
        assert_eq!(resp.status(), http::StatusCode::NO_CONTENT);

        test_app.app_state.pgdb.clear_db();
    }

    /// Test for validating that a successful token validation returns an "UNAUTHORIZED" (401) status code when the user is missing.
    ///
    /// This test verifies that the token validation endpoint works correctly when a valid access token is provided, but the associated user is not found.
    /// It performs the following steps:
    ///
    /// 1. Initializes the application state.
    /// 2. Initializes an Actix web application for testing and configures it with the application state.
    /// 3. Generates a valid access token with a random user ID.
    /// 4. Sends a GET request to the validation endpoint (/validate) with the valid access token in the "Authorization" header.
    /// 5. Verifies that the response status code is "UNAUTHORIZED" (401), indicating that the token is valid but the user is not found.
    /// 6. Reads the error response body and checks that the error message is "Invalid token".
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_successful_validate_token_user_missing() {
        use crate::handlers::auth::auth::validate;

        let test_app = TestRepo::new().await;

        let resp = test_app.call(
            "/validate",
            "/api/auth",
            validate,
            test_app.valid_authorizate(TestRequest::get(), &Uuid::new_v4()),
        ).await;
        assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);

        let resp_body: ErrorSchema = test::read_body_json(resp).await;
        assert_eq!(resp_body.message, "Invalid token");
    }

    /// Test for validating that an invalid access token returns an "UNAUTHORIZED" (401) status code.
    ///
    /// This test checks the behavior of the token validation endpoint when an invalid access token is provided. It performs the following steps:
    ///
    /// 1. Initializes the application state.
    /// 2. Creates a test user and an associated user repository.
    /// 3. Initializes an Actix web application for testing and configures it with the application state.
    /// 4. Generates an invalid access token using a known user's ID.
    /// 5. Sends a GET request to the validation endpoint (/validate) with the invalid access token in the "Authorization" header.
    /// 6. Verifies that the response status code is "UNAUTHORIZED" (401), indicating that the token is invalid.
    /// 7. Reads the error response body and checks that the error message is "Invalid token".
    /// 8. Deletes the test user.
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_validate_invalid_token() {
        use crate::handlers::auth::auth::validate;

        let test_app = TestRepo::new().await;

        let created_user = util::create_test_user(
            &RegisterUserSchema {
                name: "Test".to_string(),
                email: "Test@test.de".to_string(),
                password: "123".to_string(),
            },
            &test_app.user_repo.clone(),
        );

        let resp = test_app.call(
            "/validate",
            "/api/auth",
            validate,
            test_app.invalid_authorizate(TestRequest::get(), &created_user.id)
        ).await;
        assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);

        let resp_body: ErrorSchema = test::read_body_json(resp).await;
        assert_eq!(resp_body.message, "Invalid token");

        test_app.app_state.pgdb.clear_db();
    }

    /// Test for validating that an invalid access token in a cookie returns an "UNAUTHORIZED" (401) status code.
    ///
    /// This test checks the behavior of the token validation endpoint when an invalid access token is provided in a cookie. It performs the following steps:
    ///
    /// 1. Initializes the application state.
    /// 2. Creates a test user and an associated user repository.
    /// 3. Initializes an Actix web application for testing and configures it with the application state.
    /// 4. Generates an invalid access token using a known user's ID.
    /// 5. Constructs a cookie containing the invalid access token and sends a GET request to the validation endpoint (/validate) with the cookie.
    /// 6. Verifies that the response status code is "UNAUTHORIZED" (401), indicating that the token is invalid.
    /// 7. Reads the error response body and checks that the error message is "Invalid token".
    /// 8. Deletes the test user.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_validate_invalid_token_cookie() {
        use crate::handlers::auth::auth::validate;

        let test_app = TestRepo::new().await;

        let created_user = util::create_test_user(
            &RegisterUserSchema {
                name: "Test".to_string(),
                email: "Test@test.de".to_string(),
                password: "123".to_string(),
            },
            &test_app.user_repo.clone(),
        );

        let valid_access_token = util::create_invalid_jwt_token(&created_user.id, &test_app.app_state);

        let cookie = Cookie::build("token", valid_access_token)
            .path("/")
            .max_age(Duration::new(60 * 60, 0))
            .http_only(true)
            .finish();

        let resp = test_app.call(
            "/validate",
            "/api/auth",
            validate,
            TestRequest::get().cookie(cookie),
        ).await;
        assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);

        let resp_body: ErrorSchema = test::read_body_json(resp).await;
        assert_eq!(resp_body.message, "Invalid token");

        test_app.app_state.pgdb.clear_db();
    }

    /// Test for successfully logging out with a valid access token stored in a cookie.
    ///
    /// This test verifies the behavior of the `logout` handler when a user logs out by clearing the token stored in a cookie. The test performs the following steps:
    ///
    /// 1. Creates a test application state and a user repository.
    /// 2. Creates a test user named `created_user`.
    /// 3. Generates a valid access token for `created_user` using the test application state.
    /// 4. Builds a cookie with the access token and sends a POST request to the `logout` handler.
    /// 5. Asserts that the response status code is "NO_CONTENT" (204), indicating a successful logout.
    /// 6. Parses the "Set-Cookie" header from the response to verify that the cookie value is cleared.
    /// 7. Asserts that the token from the cookie is an empty string, confirming that the user's session has been terminated.
    ///
    /// This test ensures that the `logout` handler correctly logs out users by clearing their session cookies.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_successful_logout_with_cookie() {
        use crate::handlers::auth::auth::logout;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let valid_access_token = util::create_valid_jwt_token(&created_user.id, &test_app.app_state);

        let cookie = Cookie::build("token", valid_access_token)
            .path("/")
            .max_age(Duration::new(60 * 60, 0))
            .http_only(true)
            .finish();

        let resp = test_app.call(
            "/logout",
            "/api/auth",
            logout,
            TestRequest::post().cookie(cookie)
        ).await;
        assert_eq!(resp.status(), http::StatusCode::NO_CONTENT);


        let set_cookie_header = resp
            .headers()
            .get(actix_web::http::header::SET_COOKIE)
            .unwrap();
        let set_cookie_value = set_cookie_header.to_str().unwrap();

        let cookie = Cookie::parse(set_cookie_value).unwrap();
        let token_from_cookie = cookie.value();

        assert_eq!(token_from_cookie, "");

        test_app.app_state.pgdb.clear_db();
    }

    /// Test for an unsuccessful logout attempt with an invalid or expired access token stored in a cookie.
    ///
    /// This test verifies the behavior of the `logout` handler when a user attempts to log out with an invalid or expired access token stored in a cookie. The test performs the following steps:
    ///
    /// 1. Creates a test application state and a user repository.
    /// 2. Creates a test user named `created_user`.
    /// 3. Generates an invalid or expired access token for `created_user` using the test application state.
    /// 4. Builds a cookie with the access token and sends a POST request to the `logout` handler.
    /// 5. Asserts that the response status code is "UNAUTHORIZED" (401), indicating an unsuccessful logout.
    ///
    /// This test ensures that the `logout` handler correctly handles cases where users attempt to log out with invalid or expired tokens.
    ///
    #[actix_web::test]
    #[serial_test::serial]
    async fn test_unsuccessful_logout_with_cookie() {
        use crate::handlers::auth::auth::logout;

        let test_app = TestRepo::new().await;

        let created_user = util::create_standard_test_user(&test_app.user_repo);

        let valid_access_token = util::create_invalid_jwt_token(&created_user.id, &test_app.app_state);

        let cookie = Cookie::build("token", valid_access_token)
            .path("/")
            .max_age(Duration::new(60 * 60, 0))
            .http_only(true)
            .finish();

        let resp = test_app.call(
            "/logout",
            "/api/auth",
            logout,
            TestRequest::post().cookie(cookie)
        ).await;
        assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);
    }
}
