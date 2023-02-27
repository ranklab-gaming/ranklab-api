use super::{Auth, AuthError};
use crate::config::Config;
use crate::guards::auth::AuthFromRequest;
use crate::guards::DbConn;
use crate::models::{Coach, Player};
use diesel::prelude::*;
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use lazy_static::lazy_static;
use regex::Regex;
use rocket::{Request, State};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

lazy_static! {
  static ref JWT_REGEX: Regex = Regex::new(r"Bearer (?P<jwt>.*)").unwrap();
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct Claims {
  pub sub: String,
}

#[derive(Debug, Clone, Deserialize)]
pub enum KeyAlgorithm {
  RS256,
}

#[derive(Debug, Clone, Deserialize)]
pub enum KeyType {
  #[serde(rename = "RSA")]
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

pub struct Jwt<T>(pub T);

impl<T> Jwt<T> {
  pub fn into_inner(self) -> T {
    self.0
  }
}

impl<T> Auth<Jwt<T>> {
  pub fn into_deep_inner(self) -> T {
    self.into_inner().into_inner()
  }
}

#[async_trait]
pub trait FromJwt: Sized {
  async fn from_jwt(jwt: &Claims, db_conn: &DbConn) -> Result<Self, AuthError>;
}

#[async_trait]
impl<T: FromJwt> AuthFromRequest for Jwt<T> {
  async fn from_request(req: &Request<'_>) -> Result<Self, AuthError> {
    let config = req.guard::<&State<Config>>().await;
    let db_conn = req.guard::<DbConn>().await.unwrap();
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

    let captures = JWT_REGEX
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

    let claims = decode::<Claims>(
      jwt,
      &DecodingKey::from_rsa_components(&jwk.n, &jwk.e).unwrap(),
      &validation,
    )
    .map_err(|e| AuthError::Invalid(e.to_string()))
    .map(|data| data.claims);

    let inner = T::from_jwt(&claims?, &db_conn).await?;

    Ok(Self(inner))
  }
}

#[async_trait]
impl FromJwt for Coach {
  async fn from_jwt(jwt: &Claims, db_conn: &DbConn) -> Result<Self, AuthError> {
    let uuid_str = jwt.sub.replace("coach:", "");
    let uuid = Uuid::parse_str(&uuid_str).map_err(|e| AuthError::Invalid(e.to_string()))?;

    let coach = db_conn
      .run(move |conn| Coach::find_by_id(&uuid).first(conn))
      .await
      .map_err(|_| AuthError::NotFound("coach".to_string()))?;

    Ok(coach)
  }
}

#[async_trait]
impl FromJwt for Player {
  async fn from_jwt(jwt: &Claims, db_conn: &DbConn) -> Result<Self, AuthError> {
    let uuid_str = jwt.sub.replace("player:", "");
    let uuid = Uuid::parse_str(&uuid_str).map_err(|e| AuthError::Invalid(e.to_string()))?;

    let player = db_conn
      .run(move |conn| Player::find_by_id(&uuid).first(conn))
      .await
      .map_err(|_| AuthError::NotFound("player".to_string()))?;

    Ok(player)
  }
}
