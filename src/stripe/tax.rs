use crate::config::Config;

pub mod calculations;
pub mod transactions;

fn with_headers(request: reqwest::RequestBuilder, config: &Config) -> reqwest::RequestBuilder {
  request
    .header(
      "Stripe-Version",
      "2022-08-01;tax_calc_beta=v3;tax_txns_beta=v2",
    )
    .header("Authorization", format!("Bearer {}", config.stripe_secret))
}
