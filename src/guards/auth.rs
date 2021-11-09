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
use rocket_okapi::{
  gen::OpenApiGenerator,
  request::{OpenApiFromRequest, RequestHeaderInput},
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

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
  #[serde(rename = "kty")]
  _kty: KeyType,
  #[serde(rename = "alg")]
  _alg: KeyAlgorithm,
  kid: String,
  n: String,
  e: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OidcConfiguration {
  jwks_uri: String
}

#[derive(Debug, Clone, Deserialize)]
pub struct Jwks {
  keys: Vec<Jwk>,
}

pub struct Auth<T>(pub T);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Auth<User> {
  type Error = AuthError;

  async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
    use crate::schema::users::dsl::*;

    let jwt_regexp = Regex::new(r"Bearer (?P<jwt>.+)").unwrap();
    let config = req.guard::<&State<Config>>().await;
    let db_conn = req.guard::<DbConn>().await.unwrap();
    let auth0_domain = config.as_ref().unwrap().auth0_domain.clone();
    let oidc_configuration_url = format!("{}{}", auth0_domain, ".well-known/openid-configuration");

    let oidc_configuration = reqwest::get(&oidc_configuration_url)
    .await
    .unwrap()
    .json::<OidcConfiguration>()
    .await
    .unwrap();

    let jwks = reqwest::get(&oidc_configuration.jwks_uri)
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
    let validation = Validation::new(Algorithm::RS256);

    let decoded_jwt = try_result!(decode::<Claims>(
      &jwt,
      &DecodingKey::from_rsa_components(&jwk.n, &jwk.e).unwrap(),
      &validation
    )
    .map_err(|_| AuthError::Invalid));

    let sub = decoded_jwt.claims.sub.clone();

    let user = db_conn
      .run(|conn| {
        users
          .filter(auth0_id.eq(decoded_jwt.claims.sub))
          .first(conn)
      })
      .await;

    match user {
      Ok(user) => Outcome::Success(Auth(user)),
      Err(diesel::result::Error::NotFound) => {
        let user: User = db_conn
          .run(|conn| {
            diesel::insert_into(users)
              .values(&vec![(auth0_id.eq(sub))])
              .get_result(conn)
              .unwrap()
          })
          .await;

        Outcome::Success(Auth(user))
      }
      Err(_) => panic!("Failure creating user"),
    }
  }
}

impl<'a> OpenApiFromRequest<'a> for Auth<User> {
  fn from_request_input(
    _gen: &mut OpenApiGenerator,
    _name: String,
    _required: bool,
  ) -> rocket_okapi::Result<RequestHeaderInput> {
    Ok(RequestHeaderInput::None)
  }
}
