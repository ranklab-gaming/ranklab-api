use crate::config::Config;
use hmac::{Hmac, Mac};

pub fn generate_user_hash(email: &str, config: &Config) -> Option<String> {
  config.intercom_verification_secret.as_ref().map(|secret| {
    let mut mac = Hmac::<sha2::Sha256>::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(email.as_bytes());
    let result = mac.finalize().into_bytes();
    hex::encode(result)
  })
}
