use crate::config::Config;
use crate::db::DbConn;
use crate::models::User;
use crate::try_result;
use diesel::prelude::*;
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use regex::Regex;
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};
use rocket::State;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum AuthError {
  #[error("missing authorization header")]
  Missing,
  #[error("invalid token")]
  Invalid,
}

impl From<AuthError> for (Status, AuthError) {
  fn from(error: AuthError) -> Self {
    match error {
      AuthError::Missing => (Status::Unauthorized, error),
      AuthError::Invalid => (Status::BadRequest, error),
    }
  }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Claims {
  pub sub: String,

  #[serde(rename(
    deserialize = "https://ranklab.gg/id",
    serialize = "https://ranklab.gg/id"
  ))]
  pub id: Uuid,
}

#[derive(Debug, Clone, Deserialize)]
enum KeyAlgorithm {
  RS256,
}

#[derive(Debug, Clone, Deserialize)]
enum KeyType {
  RSA,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Jwk {
  kty: KeyType,
  alg: KeyAlgorithm,
  kid: String,
  n: String,
  e: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Jwks {
  keys: Vec<Jwk>,
}

pub struct ApiKey;
pub struct Auth<T>(pub T);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Auth<ApiKey> {
  type Error = AuthError;

  async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
    let config = req.guard::<&State<Config>>().await;
    let api_key = config.as_ref().unwrap().api_key.clone();
    let is_valid = |key: &str| -> bool { key == format!("Bearer {}", api_key) };

    match req.headers().get_one("authorization") {
      None => Outcome::Failure((Status::BadRequest, AuthError::Missing)),
      Some(key) if is_valid(key) => Outcome::Success(Auth(ApiKey)),
      Some(_) => Outcome::Failure((Status::Unauthorized, AuthError::Invalid)),
    }
  }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Auth<User> {
  type Error = AuthError;

  async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
    let jwt_regexp = Regex::new(r"Bearer (?P<jwt>.+)").unwrap();
    let config = req.guard::<&State<Config>>().await;
    let db_conn = req.guard::<DbConn>().await.unwrap();
    let auth0_domain = config.as_ref().unwrap().auth0_domain.clone();
    let jwks_url = format!("{}{}", auth0_domain, ".well-known/jwks.json");

    let jwks = reqwest::get(&jwks_url)
      .await
      .unwrap()
      .json::<Jwks>()
      .await
      .unwrap();

    let authorization = try_result!(req
      .headers()
      .get_one("authorization")
      .ok_or(AuthError::Missing));

    let captures = try_result!(jwt_regexp.captures(authorization).ok_or(AuthError::Invalid));
    let jwt = try_result!(captures.name("jwt").ok_or(AuthError::Invalid)).as_str();
    let header = try_result!(decode_header(jwt).map_err(|_| AuthError::Invalid));
    let kid = header.kid.unwrap();
    let jwk = jwks.keys.iter().find(|jwk| jwk.kid == kid).unwrap();

    let decoded_jwt = try_result!(decode::<Claims>(
      &jwt,
      &DecodingKey::from_rsa_components(&jwk.n, &jwk.e).unwrap(),
      &Validation {
        algorithms: vec![Algorithm::RS256],
        validate_exp: true,
        ..Default::default()
      },
    )
    .map_err(|_| AuthError::Invalid));

    let user: User = db_conn
      .run(move |conn| {
        use crate::schema::users::dsl::*;
        users.filter(id.eq(decoded_jwt.claims.id)).first(conn)
      })
      .await
      .unwrap();

    Outcome::Success(Auth(user))
  }
}
