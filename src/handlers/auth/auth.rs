// parts are from https://codevoweb.com/rust-jwt-authentication-with-actix-web/
// Documentation was created by ChatGPT
use actix_web::{web::{ServiceConfig, self, Json, Data}, HttpResponse, post, get, cookie::{Cookie, time::Duration as ActixWebDuration}};
use argon2::{password_hash::SaltString, Argon2, PasswordHasher, PasswordHash, PasswordVerifier};
use chrono::{Duration, Utc};
use rand_core::OsRng;
use serde_json::json;

use crate::{models::{auth::{RegisterUserSchema, LoginUserSchema, LoginResponse}, users::{NewUser, UserResponse}, jwt::TokenClaims}, repository::users::UsersRepo, AppState, jwt};
use jsonwebtoken::{encode, EncodingKey, Header};

/// # User Registration Endpoint
///
/// This endpoint allows users to register by providing necessary information.
#[utoipa::path(
    post,
    path = "/api/auth/register",
    request_body = RegisterUserSchema,
    tag = "auth",
    responses(
        (status = 201, description = "User registration successful. Returns user information.", body = UserResponse),
        (status = 409, description = "User with the specified email already exists.", body = ErrorSchema)
    )
)]
#[post("/register")]
pub async fn register(
    body: Json<RegisterUserSchema>,
    db: Data<UsersRepo>,
) -> HttpResponse {
    let exists = db.exist_user_with_email(body.email.to_lowercase()).unwrap();

    if exists {
        return HttpResponse::Conflict().json(
            json!({"message": "User with that email already exists"}),
        );
    }

    let salt = SaltString::generate(&mut OsRng);
    let hashed_password = Argon2::default()
        .hash_password(body.password.as_bytes(), &salt)
        .expect("Error while hashing password")
        .to_string();

    let user = db.create_user(&NewUser {
        password: &hashed_password,
        name: &body.name,
        email: &body.email.to_lowercase(),
    });

    match user {
        Ok(user) => HttpResponse::Created().json(UserResponse {
            id: user.id,
            email: user.email,
            name: user.name,
        }),
        Err(_) => HttpResponse::InternalServerError().json(
            json!({"message": "An unexpected error has occured"})
        ),
    }
}

/// # User Login Endpoint
///
/// This endpoint allows users to log in by providing their email and password.
#[utoipa::path(
    post,
    path = "/api/auth/login",
    request_body = LoginUserSchema,
    tag = "auth",
    responses(
        (status = 200, description = "Login successful. Returns a JWT token and user ID.", body = LoginResponse),
        (status = 401, description = "Invalid email or password.", body = ErrorSchema),
    )
)]
#[post("/login")]
pub async fn login(
    body: Json<LoginUserSchema>,
    db: Data<UsersRepo>,
    app: Data<AppState>,
) -> HttpResponse {
    let user = match db.fetch_active_user_password_by_email(body.email.to_lowercase()) {
        Ok(v) => v,
        Err(_) => return HttpResponse::Unauthorized().json(
            serde_json::json!({"message": "Invalid email"})
        )
    };

    let parsed_hash = PasswordHash::new(&user.password).unwrap();
    let is_valid = Argon2::default()
        .verify_password(body.password.as_bytes(), &parsed_hash)
        .map_or(false, |_| true);

    if !is_valid {
        return HttpResponse::Unauthorized().json(
            serde_json::json!({"message": "Invalid email or password"})
        )
    }

    let now = Utc::now();
    let iat = now.timestamp() as usize;
    let exp = (now + Duration::minutes(60)).timestamp() as usize;
    
    let claims: TokenClaims = TokenClaims { 
        sub: user.id.to_string(), 
        iat, 
        exp, 
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(app.env.jwt_secret.as_ref()),
    ).unwrap();

    let cookie = Cookie::build("token", token.to_owned())
        .path("/")
        .max_age(ActixWebDuration::new(60 * 60, 0))
        .http_only(true)
        .finish();

    HttpResponse::Ok()
        .cookie(cookie)
        .json(LoginResponse {
            token,
            user_id: user.id,
        })
}

/// # User Logout Endpoint
///
/// This endpoint allows users to log out, clearing the authentication token cookie.
#[utoipa::path(
    post,
    path = "/api/auth/logout",
    tag = "auth",
    responses(
        (status = 204, description = "Logout successful. Clears the authentication")
    )
)]
#[post("/logout")]
pub async fn logout(_: jwt::JwtMiddleware) -> HttpResponse {
    let cookie = Cookie::build("token", "")
        .path("/")
        .max_age(ActixWebDuration::new(-1, 0))
        .http_only(true)
        .finish();

    HttpResponse::NoContent()
        .cookie(cookie)
        .finish()
}

/// # User Token Validation Endpoint
///
/// This endpoint allows validating the authenticity of a user's token.
#[utoipa::path(
    get,
    path = "/api/auth/validate",
    tag = "auth",
    responses(
        (status = 204, description = "Token is valid.")
    )
)]
#[get("/validate")]
pub async fn validate(_: jwt::JwtMiddleware) -> HttpResponse {
    HttpResponse::NoContent()
        .finish()
}

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .service(register)
            .service(login)
            .service(logout)
            .service(validate)
    );
}