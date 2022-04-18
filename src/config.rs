use rocket::figment::Profile;
use serde::{Deserialize, Serialize};

pub const DEVELOPMENT_PROFILE: Profile = Profile::const_new("development");
pub const TEST_PROFILE: Profile = Profile::const_new("test");
pub const PRODUCTION_PROFILE: Profile = Profile::const_new("production");

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
  pub auth0_issuer_base_url: String,
  pub s3_bucket: String,
  pub s3_bucket_queue: String,
  pub aws_access_key_id: String,
  pub aws_secret_key: String,
  pub sentry_dsn: Option<String>,
  pub stripe_secret: String,
  pub stripe_direct_webhooks_queue: String,
  pub stripe_direct_webhooks_secret: String,
  pub stripe_connect_webhooks_queue: String,
  pub stripe_connect_webhooks_secret: String,
  pub stripe_product_id: String,
}
