use crate::{config::Config, stripe::RequestError};
use schemars::JsonSchema;
use serde::Deserialize;

use super::with_headers;

#[derive(Deserialize, JsonSchema)]
pub struct TaxTransaction {
  pub id: String,
}

impl TaxTransaction {
  pub async fn create_reversal(
    config: &Config,
    tax_transaction_id: String,
    reference: String,
  ) -> Result<Self, RequestError> {
    let client = reqwest::Client::new();
    let request = client.post("https://api.stripe.com/v1/tax/transactions/create_reversal");

    let body = [
      ("reversal[original_transaction]", tax_transaction_id),
      ("reference", reference),
      ("mode", "full".to_string()),
    ];

    let response = with_headers(request, config).form(&body).send().await?;

    let tax_transaction = match response.error_for_status() {
      Ok(response) => response.json::<TaxTransaction>().await.unwrap(),
      Err(err) => {
        if err.status() == Some(reqwest::StatusCode::BAD_REQUEST) {
          return Err(RequestError::BadRequest(err));
        }

        return Err(err.into());
      }
    };

    Ok(tax_transaction)
  }

  pub async fn create_from_calculation(
    config: &Config,
    tax_calculation_id: String,
    reference: String,
  ) -> Result<Self, RequestError> {
    let client = reqwest::Client::new();
    let request = client.post("https://api.stripe.com/v1/tax/transactions/create_from_calculation");
    let body = [
      ("calculation", tax_calculation_id.clone()),
      ("reference", reference),
    ];

    let response = with_headers(request, config)
      .header("Idempotency-Key", tax_calculation_id)
      .form(&body)
      .send()
      .await?;

    let tax_transaction = match response.error_for_status() {
      Ok(response) => response.json::<TaxTransaction>().await.unwrap(),
      Err(err) => {
        if err.status() == Some(reqwest::StatusCode::BAD_REQUEST) {
          return Err(RequestError::BadRequest(err));
        }

        return Err(err.into());
      }
    };

    Ok(tax_transaction)
  }
}
