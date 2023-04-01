use crate::config::Config;
use hmac::{Hmac, Mac};

pub mod contacts;

pub fn generate_user_hash(email: &str, config: &Config) -> Option<String> {
  config.intercom_verification_secret.as_ref().map(|secret| {
    let mut mac = Hmac::<sha2::Sha256>::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(email.as_bytes());
    let result = mac.finalize().into_bytes();
    hex::encode(result)
  })
}

pub fn build_request(request: reqwest::RequestBuilder, config: &Config) -> reqwest::RequestBuilder {
  request
    .header("Intercom-Version", "2.8")
    .header(
      "Authorization",
      format!("Bearer {}", config.intercom_access_token.as_ref().unwrap()),
    )
    .header("Accept", "application/json")
    .header("Content-Type", "application/json")
}

#[derive(thiserror::Error, Debug)]
pub enum RequestError {
  #[error("Conflict: {0}")]
  Conflict(reqwest::Error),
  #[error(transparent)]
  ServerError(#[from] reqwest::Error),
}
