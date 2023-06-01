use crate::config::Config;
use crate::guards::DbConn;
use crate::models::{Coach, Player};
use bcrypt::verify;
use chrono::{Duration, Utc};
use diesel::prelude::*;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use rocket::State;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, JsonSchema, Copy, Clone, FromFormField)]
#[serde(rename_all = "snake_case")]
pub enum UserType {
  Coach,
  Player,
}

pub enum Account {
  Player(Player),
  Coach(Coach),
}

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

pub fn generate_token(account: &Account, config: &Config) -> String {
  let now = Utc::now();
  let exp = (now + Duration::minutes(1)).timestamp() as usize;

  let sub = match account {
    Account::Coach(coach) => format!("coach:{}", coach.id),
    Account::Player(player) => format!("player:{}", player.id),
  };

  let claims = Claims {
    sub,
    exp,
    iss: config.host.clone(),
  };

  let key = EncodingKey::from_secret(config.auth_client_secret.as_ref());
  encode(&Header::default(), &claims, &key).unwrap()
}

pub async fn create_with_password(
  user_type: UserType,
  credentials: &PasswordCredentials,
  config: &State<Config>,
  db_conn: DbConn,
) -> Option<String> {
  let session_password = credentials.password.clone();
  let email = credentials.email.clone();

  let account = match user_type {
    UserType::Coach => Account::Coach(
      db_conn
        .run(move |conn| Coach::find_by_email(&email).get_result::<Coach>(conn))
        .await
        .ok()?,
    ),
    UserType::Player => Account::Player(
      db_conn
        .run(move |conn| Player::find_by_email(&email).get_result::<Player>(conn))
        .await
        .ok()?,
    ),
  };

  let password = match &account {
    Account::Coach(coach) => coach.password.clone(),
    Account::Player(player) => player.password.clone(),
  };

  match password {
    Some(password) => {
      let valid = verify(session_password, &password).ok()?;

      if !valid {
        return None;
      }
    }
    None => return None,
  }

  Some(generate_token(&account, config))
}

pub fn decode_token_credentials(
  credentials: &TokenCredentials,
  config: &Config,
) -> Option<CredentialsClaims> {
  let token = credentials.token.clone();
  let mut validation = Validation::new(Algorithm::HS256);

  validation.set_issuer(&[config.web_host.clone()]);
  validation.validate_exp = true;

  let jwt = decode::<CredentialsClaims>(
    &token,
    &DecodingKey::from_secret(config.auth_client_secret.as_ref()),
    &validation,
  )
  .ok()?;

  Some(jwt.claims)
}

pub async fn create_with_token(
  user_type: UserType,
  credentials: &TokenCredentials,
  config: &State<Config>,
  db_conn: DbConn,
) -> Option<String> {
  let claims = decode_token_credentials(credentials, config)?;
  let email = claims.sub;

  let account = match user_type {
    UserType::Coach => Account::Coach(
      db_conn
        .run(move |conn| Coach::find_by_email(&email).get_result::<Coach>(conn))
        .await
        .ok()?,
    ),
    UserType::Player => Account::Player(
      db_conn
        .run(move |conn| Player::find_by_email(&email).get_result::<Player>(conn))
        .await
        .ok()?,
    ),
  };

  Some(generate_token(&account, config))
}
