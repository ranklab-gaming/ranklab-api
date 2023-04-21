use crate::{
  config::Config,
  stripe::{build_request, RequestError},
};
use schemars::JsonSchema;
use serde::Deserialize;
use stripe::CustomerId;

#[derive(Deserialize, JsonSchema)]
pub struct TaxCalculation {
  pub id: String,
  pub tax_amount_exclusive: i64,
  pub amount_total: i64,
}

impl TaxCalculation {
  pub async fn create(
    config: &Config,
    customer_id: &CustomerId,
    price: i64,
  ) -> Result<Self, RequestError> {
    let client = reqwest::Client::new();
    let request = client.post("https://api.stripe.com/v1/tax/calculations");

    let body = [
      ("currency", "usd".to_string()),
      ("customer", customer_id.to_string()),
      ("line_items[0][amount]", price.to_string()),
      ("line_items[0][reference]", "0".to_string()),
    ];

    let response = build_request(request, config).form(&body).send().await?;

    let tax_calculation = match response.error_for_status() {
      Ok(response) => response.json::<TaxCalculation>().await.unwrap(),
      Err(err) => {
        if err.status() == Some(reqwest::StatusCode::BAD_REQUEST) {
          return Err(RequestError::BadRequest(err));
        }

        return Err(err.into());
      }
    };

    Ok(tax_calculation)
  }

  pub async fn retrieve(config: &Config, tax_calculation_id: String) -> Result<Self, RequestError> {
    let client = reqwest::Client::new();

    let request = client.get(format!(
      "https://api.stripe.com/v1/tax/calculations/{}",
      tax_calculation_id
    ));

    let response = build_request(request, config).send().await?;
    let json = response.json::<TaxCalculation>().await?;

    Ok(json)
  }
}
