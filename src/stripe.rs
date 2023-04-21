mod tax;
pub use tax::calculations::{TaxCalculation, TaxCalculationLineItem};
pub use tax::transactions::TaxTransaction;

use crate::config::Config;

#[derive(thiserror::Error, Debug)]
pub enum RequestError {
  #[error("Bad request: {0}")]
  BadRequest(reqwest::Error),
  #[error(transparent)]
  ServerError(#[from] reqwest::Error),
  #[error("Not found: {0}")]
  NotFound(reqwest::Error),
}

pub fn build_request(request: reqwest::RequestBuilder, config: &Config) -> reqwest::RequestBuilder {
  request
    .header("Stripe-Version", "2022-11-15")
    .header("Authorization", format!("Bearer {}", config.stripe_secret))
}
