use crate::config::Config;
use crate::guards::DbConn;
use crate::models::{Coach, OneTimeToken, Player};
use crate::try_result;
use diesel::prelude::*;
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use okapi::openapi3::*;
use regex::Regex;
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};
use rocket::State;
use rocket_okapi::gen::OpenApiGenerator;
use rocket_okapi::request::{OpenApiFromRequest, RequestHeaderInput};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

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

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, JsonSchema, Copy, Clone)]
#[serde(rename_all = "snake_case")]
pub enum UserType {
  Coach,
  Player,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct Claims {
  pub sub: String,
}

impl Claims {
  pub fn user_type(&self) -> UserType {
    if self.sub.to_string().starts_with("coach:") {
      UserType::Coach
    } else if self.sub.to_string().starts_with("player:") {
      UserType::Player
    } else {
      panic!("invalid sub: {}", self.sub)
    }
  }
}

#[derive(Debug, Clone, Deserialize)]
enum KeyAlgorithm {
  RS256,
}

#[derive(Debug, Clone, Deserialize)]
enum KeyType {
  Rsa,
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
  issuer: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Jwks {
  keys: Vec<Jwk>,
}

pub struct Auth<T>(pub T);

async fn decode_jwt<'r>(req: &'r Request<'_>) -> Result<Claims, AuthError> {
  let jwt_regexp = Regex::new(r"Bearer (?P<jwt>.+)").unwrap();
  let config = req.guard::<&State<Config>>().await;
  let web_host = config.as_ref().unwrap().web_host.clone();

  let oidc_configuration_url = format!(
    "{}{}",
    web_host, "/api/oidc/.well-known/openid-configuration"
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
    .ok_or_else(|| AuthError::Invalid("malformed authorization header".to_string()))?;

  let jwt = captures
    .name("jwt")
    .ok_or_else(|| AuthError::Invalid("jwt not found in header".to_string()))?
    .as_str();

  let header = decode_header(jwt).map_err(|e| AuthError::Invalid(e.to_string()))?;
  let kid = header.kid.unwrap();
  let jwk = jwks.keys.iter().find(|jwk| jwk.kid == kid).unwrap();
  let mut validation = Validation::new(Algorithm::RS256);

  validation.set_issuer(&[oidc_configuration.issuer]);

  decode::<Claims>(
    jwt,
    &DecodingKey::from_rsa_components(&jwk.n, &jwk.e).unwrap(),
    &validation,
  )
  .map_err(|e| AuthError::Invalid(e.to_string()))
  .map(|data| data.claims)
}

impl Auth<Coach> {
  async fn from_jwt(db_conn: DbConn, jwt: Claims) -> Result<Self, AuthError> {
    if jwt.user_type() != UserType::Coach {
      return Err(AuthError::Invalid("not a coach".to_string()));
    }

    let uuid_str = jwt.sub.replace("coach:", "");
    let uuid = Uuid::parse_str(&uuid_str).map_err(|e| AuthError::Invalid(e.to_string()))?;

    let coach = db_conn
      .run(move |conn| Coach::find_by_id(&uuid).first(conn))
      .await
      .map_err(|_| AuthError::NotFound("coach".to_string()))?;

    Ok(Auth(coach))
  }
}

impl Auth<Player> {
  async fn from_jwt(db_conn: DbConn, jwt: Claims) -> Result<Self, AuthError> {
    if jwt.user_type() != UserType::Player {
      return Err(AuthError::Invalid("not a player".to_string()));
    }

    let uuid_str = jwt.sub.replace("player:", "");
    let uuid = Uuid::parse_str(&uuid_str).map_err(|e| AuthError::Invalid(e.to_string()))?;

    let player = db_conn
      .run(move |conn| Player::find_by_id(&uuid).first(conn))
      .await
      .map_err(|_| AuthError::NotFound("player".to_string()))?;

    Ok(Auth(player))
  }
}

impl Auth<OneTimeToken> {
  async fn from_req<'r>(req: &'r Request<'_>) -> Result<Self, AuthError> {
    let db_conn = req.guard::<DbConn>().await.unwrap();

    let value = match req.query_value::<String>("auth[token]") {
      Some(Ok(token)) => token,
      _ => return Err(AuthError::Missing),
    };

    let user_type: UserType = match req.query_value::<String>("auth[user_type]") {
      Some(Ok(user_type)) => serde_json::from_str(&user_type).unwrap(),
      _ => return Err(AuthError::Missing),
    };

    let token = db_conn
      .run(move |conn| {
        OneTimeToken::find_by_value(&value, user_type, "reset-password").first::<OneTimeToken>(conn)
      })
      .await
      .map_err(|_| AuthError::NotFound("token".to_string()))?;

    Ok(Auth(token))
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
impl<'r> FromRequest<'r> for Auth<OneTimeToken> {
  type Error = AuthError;

  async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
    let auth = try_result!(Auth::<OneTimeToken>::from_req(req).await);
    Outcome::Success(auth)
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

#[allow(dead_code)]
#[derive(Debug, Deserialize, JsonSchema)]
struct AuthQuery {
  token: String,
  user_type: UserType,
}

impl<'a> OpenApiFromRequest<'a> for Auth<OneTimeToken> {
  fn from_request_input(
    gen: &mut OpenApiGenerator,
    _name: String,
    required: bool,
  ) -> rocket_okapi::Result<RequestHeaderInput> {
    let schema = gen.json_schema::<AuthQuery>();
    Ok(RequestHeaderInput::Parameter(Parameter {
      name: "auth".to_owned(),
      location: "query".to_owned(),
      description: None,
      required,
      deprecated: false,
      allow_empty_value: false,
      value: ParameterValue::Schema {
        style: None,
        explode: None,
        allow_reserved: false,
        schema,
        example: None,
        examples: None,
      },
      extensions: Object::default(),
    }))
  }
}
