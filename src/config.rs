use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
  pub auth_client_secret: String,
  pub host: String,
  pub web_host: String,
  pub s3_bucket: String,
  pub s3_bucket_queue: String,
  pub aws_access_key_id: String,
  pub aws_secret_key: String,
  pub sentry_dsn: Option<String>,
  pub intercom_access_token: Option<String>,
  pub intercom_verification_secret: Option<String>,
  pub instance_id: Option<String>,
}
