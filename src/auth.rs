use crate::config::Config;
use crate::models::User;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
  sub: String,
  exp: usize,
  iss: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CredentialsClaims {
  pub sub: String,
}

#[derive(Debug, Deserialize, JsonSchema, Validate)]
pub struct PasswordCredentials {
  #[validate(email)]
  pub email: String,
  #[validate(length(min = 8))]
  pub password: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct TokenCredentials {
  pub token: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Credentials {
  Password(PasswordCredentials),
  Token(TokenCredentials),
}

pub fn generate_token(user: &User, config: &Config) -> String {
  let now = Utc::now();
  let exp = (now + Duration::minutes(1)).timestamp() as usize;
  let sub = user.id.to_string();

  let claims = Claims {
    sub,
    exp,
    iss: config.host.clone(),
  };

  let key = EncodingKey::from_secret(config.auth_client_secret.as_ref());
  encode(&Header::default(), &claims, &key).unwrap()
}

pub fn decode_token_credentials(
  credentials: &TokenCredentials,
  config: &Config,
) -> Option<CredentialsClaims> {
  let token = credentials.token.clone();
  let mut validation = Validation::new(Algorithm::HS256);

  validation.set_issuer(&[config.web_host.clone()]);
  validation.set_audience(&[config.host.clone()]);
  validation.validate_exp = true;

  let jwt = decode::<CredentialsClaims>(
    &token,
    &DecodingKey::from_secret(config.auth_client_secret.as_ref()),
    &validation,
  )
  .ok()?;

  Some(jwt.claims)
}
