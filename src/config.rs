use rocket::serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
  pub api_key: String,
  pub auth0_domain: String,
}
