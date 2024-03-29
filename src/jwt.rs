// from https://codevoweb.com/rust-jwt-authentication-with-actix-web/
use std::future::{ready, Ready};

use actix_web::error::ErrorUnauthorized;
use actix_web::{dev::Payload, Error as ActixWebError};
use actix_web::{http, web, FromRequest, HttpMessage, HttpRequest};
use jsonwebtoken::{decode, DecodingKey, Validation};
use uuid::Uuid;

use crate::AppState;
use crate::models::jwt::TokenClaims;
use crate::models::util::ErrorSchema;


pub struct JwtMiddleware {
    pub user_id: Uuid,
}

impl FromRequest for JwtMiddleware {
    type Error = ActixWebError;
    type Future = Ready<Result<Self, Self::Error>>;
    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let data = req.app_data::<web::Data<AppState>>().unwrap();

        let token = req
            .cookie("token")
            .map(|c| c.value().to_string())
            .or_else(|| {
                req.headers()
                    .get(http::header::AUTHORIZATION)
                    .map(|h| h.to_str().unwrap().split_at(7).1.to_string())
            });

        if token.is_none() {
            let json_error = ErrorSchema {
                message: "Invalid token".to_string(),
            };
            return ready(Err(ErrorUnauthorized(json_error)));
        }

        let claims = match decode::<TokenClaims>(
            &token.unwrap(),
            &DecodingKey::from_secret(data.env.jwt_secret.as_ref()),
            &Validation::default(),
        ) {
            Ok(c) => c.claims,
            Err(_) => {
                let json_error = ErrorSchema {
                    message: "Invalid token".to_string(),
                };
                return ready(Err(ErrorUnauthorized(json_error)));
            }
        };

        let user_id = Uuid::parse_str(&claims.sub).unwrap();

        if !data.pgdb.user_exists(&user_id) {
            let json_error = ErrorSchema {
                message: "Invalid token".to_string(),
            };
            return ready(Err(ErrorUnauthorized(json_error)));
        }

        req.extensions_mut()
            .insert::<Uuid>(user_id);

        ready(Ok(JwtMiddleware { user_id }))
    }
}