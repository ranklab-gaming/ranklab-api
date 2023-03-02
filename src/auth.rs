use crate::config::Config;
use crate::models::{Coach, Player};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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
