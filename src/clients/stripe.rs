pub struct StripeClient(stripe::Client);
use crate::config::Config;

impl StripeClient {
  pub fn new(config: &Config) -> Self {
    let stripe_secret = config.stripe_secret.clone();
    Self(stripe::Client::new(stripe_secret))
  }
}

impl StripeClient {
  pub fn into_inner(self) -> stripe::Client {
    self.0
  }
}

impl AsRef<stripe::Client> for StripeClient {
  fn as_ref(&self) -> &stripe::Client {
    &self.0
  }
}
