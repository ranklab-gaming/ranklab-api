use crate::config::Config;

pub mod calculations;
pub mod transactions;

struct Request;

impl Request {
  fn with_headers(request: reqwest::RequestBuilder, config: &Config) -> reqwest::RequestBuilder {
    request
      .header("Stripe-Version", "2022-11-15")
      .header("Authorization", format!("Bearer {}", config.stripe_secret))
  }
}
