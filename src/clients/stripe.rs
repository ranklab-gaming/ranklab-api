pub struct StripeClient(pub stripe::Client);
use stripe::{ApiVersion, Headers};

use crate::config::Config;

impl StripeClient {
  pub fn new(config: &Config) -> Self {
    let stripe_secret = config.stripe_secret.clone();
    let client = stripe::Client::new(stripe_secret);

    let client = client.with_headers(Headers {
      stripe_version: Some(ApiVersion::V2020_08_27_OrdersV2),
      ..Default::default()
    });

    Self(client)
  }
}
