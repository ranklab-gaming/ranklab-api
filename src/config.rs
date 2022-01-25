use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
  pub auth0_issuer_base_url: String,
  pub s3_bucket: String,
  pub s3_bucket_queue: String,
  pub aws_access_key_id: String,
  pub aws_secret_key: String,
  pub sentry_dsn: Option<String>,
  pub stripe_secret: String,
  pub stripe_webhooks_queue: String,
  pub stripe_webhooks_secret: String,
}
