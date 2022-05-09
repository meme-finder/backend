use actix_web::cookie::Cookie;
use actix_web::error::ErrorUnauthorized;
use actix_web::Error as ActixError;
use actix_web::{dev::Payload, FromRequest, HttpRequest, HttpResponse};
use actix_web::{post, web, Responder};

use serde::Deserialize;

use std::env;
use std::error::Error;
use std::future::Future;
use std::pin::Pin;
use std::str::FromStr;

#[derive(Deserialize)]
pub struct LoginRequest {
    pub token: String,
}

#[post("/login")]
async fn login(login_request: web::Json<LoginRequest>) -> Result<impl Responder, Box<dyn Error>> {
    let admin_token = env::var("ADMIN_TOKEN").unwrap_or_else(|_| String::from("token"));
    if login_request.0.token == admin_token {
        let cookie_secure = bool::from_str(
            env::var("COOKIE_SECURE")
                .unwrap_or_else(|_| String::from("false"))
                .as_str(),
        )?;
        let auth_cookie = Cookie::build("auth", login_request.0.token)
            .path("/")
            .secure(cookie_secure)
            .http_only(true)
            .finish();
        Ok(HttpResponse::Ok().cookie(auth_cookie).finish())
    } else {
        Err(ErrorUnauthorized("Invalid token").into())
    }
}

#[derive(Debug, Deserialize)]
pub struct AuthData {
    pub admin: bool,
}

impl FromRequest for AuthData {
    type Error = ActixError;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _pl: &mut Payload) -> Self::Future {
        let auth_cookie = req
            .cookie("auth")
            .unwrap_or_else(|| Cookie::new("auth", "no"));
        let auth = auth_cookie.value();
        let admin_token = env::var("ADMIN_TOKEN").unwrap_or_else(|_| String::from("token"));
        let auth_data = if auth == admin_token {
            Ok(AuthData { admin: true })
        } else {
            Ok(AuthData { admin: false })
        };
        Box::pin(async move { auth_data })
    }
}

pub struct NeedAuth {}

impl FromRequest for NeedAuth {
    type Error = ActixError;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _pl: &mut Payload) -> Self::Future {
        let auth_data = AuthData::extract(req);

        Box::pin(async move {
            let need_auth = if auth_data.await?.admin {
                Ok(NeedAuth {})
            } else {
                Err(ErrorUnauthorized("You are not admin"))
            };
            need_auth
        })
    }
}
