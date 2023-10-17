use super::{Auth, AuthError};
use crate::guards::DbConn;
use crate::models::User;
use crate::oidc::OidcCache;
use crate::{config::Config, guards::auth::AuthFromRequest};
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

pub struct Jwt(User);

impl Jwt {
  pub fn into_user(self) -> User {
    self.0
  }
}

impl Auth<Jwt> {
  pub fn into_user(self) -> User {
    self.0.into_user()
  }
}

impl Auth<Option<Jwt>> {
  pub fn into_user(self) -> Option<User> {
    self.0.map(|jwt| jwt.into_user())
  }
}

#[async_trait]
impl AuthFromRequest for Jwt {
  async fn from_request(req: &Request<'_>) -> Result<Self, AuthError> {
    let db_conn = req.guard::<DbConn>().await.unwrap();
    let config = req.guard::<&State<Config>>().await.unwrap();
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
    validation.set_audience(&[config.web_host.clone()]);

    let claims = decode::<Claims>(
      jwt,
      &DecodingKey::from_rsa_components(&jwk.n, &jwk.e).unwrap(),
      &validation,
    )
    .map_err(|e| AuthError::Invalid(e.to_string()))
    .map(|data| data.claims);

    let inner = User::from_jwt(&claims?, &db_conn).await?;

    Ok(Self(inner))
  }
}

impl User {
  async fn from_jwt(jwt: &Claims, db_conn: &DbConn) -> Result<Self, AuthError> {
    let uuid = Uuid::parse_str(&jwt.sub).map_err(|e| AuthError::Invalid(e.to_string()))?;

    let user = db_conn
      .run(move |conn| User::find_by_id(&uuid).first(conn))
      .await
      .map_err(|_| AuthError::NotFound)?;

    Ok(user)
  }
}
