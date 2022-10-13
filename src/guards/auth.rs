use crate::config::Config;
use crate::guards::DbConn;
use crate::models::{Coach, Player, User};
use crate::try_result;
use diesel::prelude::*;
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use regex::Regex;
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};
use rocket::State;
use rocket_okapi::gen::OpenApiGenerator;
use rocket_okapi::request::{OpenApiFromRequest, RequestHeaderInput};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AuthError {
  #[error("missing authorization header")]
  Missing,
  #[error("invalid token: {0}")]
  Invalid(String),
  #[error("not found: {0}")]
  NotFound(String),
}

impl From<AuthError> for (Status, AuthError) {
  fn from(error: AuthError) -> Self {
    match error {
      AuthError::Missing => (Status::Unauthorized, error),
      AuthError::Invalid(_) => (Status::BadRequest, error),
      AuthError::NotFound(_) => (Status::NotFound, error),
    }
  }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum UserType {
  Coach,
  Player,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Claims {
  pub sub: String,
  #[serde(rename = "https://ranklab.gg/email")]
  pub email: String,
  #[serde(rename = "https://ranklab.gg/user_type")]
  pub user_type: UserType,
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
  jwks_uri: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Jwks {
  keys: Vec<Jwk>,
}

pub struct Auth<T>(pub T);

async fn decode_jwt<'r>(req: &'r Request<'_>) -> Result<Claims, AuthError> {
  let jwt_regexp = Regex::new(r"Bearer (?P<jwt>.+)").unwrap();
  let config = req.guard::<&State<Config>>().await;
  let auth0_issuer_base_url = config.as_ref().unwrap().auth0_issuer_base_url.clone();

  let oidc_configuration_url = format!(
    "{}{}",
    auth0_issuer_base_url, ".well-known/openid-configuration"
  );

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

  let authorization = req
    .headers()
    .get_one("authorization")
    .ok_or(AuthError::Missing)?;

  let captures = jwt_regexp
    .captures(authorization)
    .ok_or(AuthError::Invalid(
      "malformed authorization header".to_string(),
    ))?;

  let jwt = captures
    .name("jwt")
    .ok_or(AuthError::Invalid("jwt not found in header".to_string()))?
    .as_str();

  let header = decode_header(jwt).map_err(|e| AuthError::Invalid(e.to_string()))?;
  let kid = header.kid.unwrap();
  let jwk = jwks.keys.iter().find(|jwk| jwk.kid == kid).unwrap();
  let validation = Validation::new(Algorithm::RS256);

  decode::<Claims>(
    &jwt,
    &DecodingKey::from_rsa_components(&jwk.n, &jwk.e).unwrap(),
    &validation,
  )
  .map_err(|e| AuthError::Invalid(e.to_string()))
  .map(|data| data.claims)
}

impl Auth<Coach> {
  async fn from_jwt(db_conn: DbConn, jwt: Claims) -> Result<Self, AuthError> {
    let coach = db_conn
      .run(|conn| Coach::find_by_auth0_id(jwt.sub).first(conn))
      .await
      .map_err(|_| AuthError::NotFound("coach".to_string()))?;

    Ok(Auth(coach))
  }
}

impl Auth<Player> {
  async fn from_jwt(db_conn: DbConn, jwt: Claims) -> Result<Self, AuthError> {
    let player = db_conn
      .run(|conn| Player::find_by_auth0_id(jwt.sub).first(conn))
      .await
      .map_err(|_| AuthError::NotFound("player".to_string()))?;

    Ok(Auth(player))
  }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Auth<Player> {
  type Error = AuthError;

  async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
    let decoded_jwt = try_result!(decode_jwt(req).await);
    let db_conn = req.guard::<DbConn>().await.unwrap();
    let auth = try_result!(Auth::<Player>::from_jwt(db_conn, decoded_jwt).await);
    Outcome::Success(auth)
  }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Auth<Coach> {
  type Error = AuthError;

  async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
    let decoded_jwt = try_result!(decode_jwt(req).await);
    let db_conn = req.guard::<DbConn>().await.unwrap();
    let auth = try_result!(Auth::<Coach>::from_jwt(db_conn, decoded_jwt).await);
    Outcome::Success(auth)
  }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Auth<Claims> {
  type Error = AuthError;

  async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
    let decoded_jwt = try_result!(decode_jwt(req).await);
    Outcome::Success(Auth(decoded_jwt))
  }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Auth<User> {
  type Error = AuthError;

  async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
    let db_conn = req.guard::<DbConn>().await.unwrap();
    let decoded_jwt = try_result!(decode_jwt(req).await);

    let user: User = match decoded_jwt.user_type {
      UserType::Player => {
        let player = try_result!(Auth::<Player>::from_jwt(db_conn, decoded_jwt).await);
        User::Player(player.0)
      }
      UserType::Coach => {
        let coach = try_result!(Auth::<Coach>::from_jwt(db_conn, decoded_jwt).await);
        User::Coach(coach.0)
      }
    };

    Outcome::Success(Auth(user))
  }
}

impl<'a> OpenApiFromRequest<'a> for Auth<Player> {
  fn from_request_input(
    _gen: &mut OpenApiGenerator,
    _name: String,
    _required: bool,
  ) -> rocket_okapi::Result<RequestHeaderInput> {
    Ok(RequestHeaderInput::None)
  }
}

impl<'a> OpenApiFromRequest<'a> for Auth<Coach> {
  fn from_request_input(
    _gen: &mut OpenApiGenerator,
    _name: String,
    _required: bool,
  ) -> rocket_okapi::Result<RequestHeaderInput> {
    Ok(RequestHeaderInput::None)
  }
}

impl<'a> OpenApiFromRequest<'a> for Auth<Claims> {
  fn from_request_input(
    _gen: &mut OpenApiGenerator,
    _name: String,
    _required: bool,
  ) -> rocket_okapi::Result<RequestHeaderInput> {
    Ok(RequestHeaderInput::None)
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
