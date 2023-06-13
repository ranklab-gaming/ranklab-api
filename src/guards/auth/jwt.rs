use super::{Auth, AuthError};
use crate::auth::{Account, UserType};
use crate::guards::auth::AuthFromRequest;
use crate::guards::DbConn;
use crate::models::{Coach, Player};
use crate::oidc::OidcCache;
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

pub struct Jwt<T>(T);

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
    let db_conn = req.guard::<DbConn>().await.unwrap();
    let oidc_cache = req.guard::<&State<OidcCache>>().await;
    let oidc_configuration = oidc_cache.as_ref().unwrap().oidc_configuration.clone();
    let jwks = oidc_cache.as_ref().unwrap().jwks.clone();

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

#[async_trait]
impl FromJwt for Account {
  async fn from_jwt(jwt: &Claims, db_conn: &DbConn) -> Result<Self, AuthError> {
    let user_type_str = jwt.sub.split(':').next().unwrap();

    let user_type =
      serde::Deserialize::deserialize(&serde_json::Value::String(user_type_str.to_string()))
        .unwrap();

    match user_type {
      UserType::Coach => Ok(Account::Coach(Coach::from_jwt(jwt, db_conn).await?)),
      UserType::Player => Ok(Account::Player(Player::from_jwt(jwt, db_conn).await?)),
    }
  }
}
