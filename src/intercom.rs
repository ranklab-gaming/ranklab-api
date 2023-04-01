use crate::config::Config;
use hmac::{Hmac, Mac};
use uuid::Uuid;

pub mod contacts;

pub fn generate_user_hash(id: Uuid, secret: &str) -> String {
  let mut mac = Hmac::<sha2::Sha256>::new_from_slice(secret.as_bytes()).unwrap();
  mac.update(id.to_string().as_bytes());
  let result = mac.finalize().into_bytes();
  hex::encode(result)
}

struct Request;

impl Request {
  fn with_headers(request: reqwest::RequestBuilder, config: &Config) -> reqwest::RequestBuilder {
    request
      .header("Intercom-Version", "2.8")
      .header(
        "Authorization",
        format!("Bearer {}", config.intercom_access_token.as_ref().unwrap()),
      )
      .header("Accept", "application/json")
      .header("Content-Type", "application/json")
  }
}
