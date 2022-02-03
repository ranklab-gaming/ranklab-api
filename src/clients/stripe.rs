pub struct StripeClient(pub stripe::Client);
use crate::config::Config;

impl StripeClient {
  pub fn new(config: &Config) -> Self {
    let stripe_secret = config.stripe_secret.clone();

    Self(stripe::Client::new(stripe_secret))
  }
}
