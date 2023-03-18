use crate::config::Config;
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Deserialize, JsonSchema, Clone)]
pub struct Address {
  pub city: String,
  pub country: String,
  pub line1: String,
  pub line2: String,
  pub postal_code: String,
  pub state: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct CustomerDetails {
  pub address: Option<Address>,
  pub ip_address: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct TaxCalculation {
  pub id: String,
  pub tax_amount_exclusive: i64,
}

#[derive(Deserialize, JsonSchema, Clone, Copy)]
pub struct TaxCalculationLineItem {
  pub amount_tax: i64,
}

#[derive(Deserialize, JsonSchema)]
struct TaxCalculationLineItemResponse {
  data: Vec<TaxCalculationLineItem>,
}

pub struct CreateTaxCalculation {
  pub customer_details: Option<CustomerDetails>,
  pub customer: Option<String>,
  pub price: i64,
}

#[derive(thiserror::Error, Debug)]
pub enum TaxCalculationError {
  #[error("Bad request")]
  BadRequest,
  #[error(transparent)]
  ServerError(#[from] reqwest::Error),
}

fn with_headers(request: reqwest::RequestBuilder, config: &Config) -> reqwest::RequestBuilder {
  request
    .header(
      "Stripe-Version",
      "2022-08-01;tax_calc_beta=v3;tax_txns_beta=v2",
    )
    .header("Authorization", format!("Bearer {}", config.stripe_secret))
}

impl TaxCalculation {
  pub async fn create(
    config: &Config,
    params: CreateTaxCalculation,
  ) -> Result<Self, TaxCalculationError> {
    let client = reqwest::Client::new();
    let request = client.post("https://api.stripe.com/v1/tax/calculations");

    let mut body = [
      ("currency", "usd".to_string()),
      ("line_items[0][amount]", params.price.to_string()),
      ("line_items[0][reference]", "0".to_string()),
    ]
    .to_vec();

    if let Some(customer) = params.customer {
      body.push(("customer", customer));
    }

    if let Some(customer_details) = params.customer_details {
      if let Some(addr) = customer_details.address {
        body.extend_from_slice(&[
          ("customer_details[address_source]", "billing".to_string()),
          ("customer_details[address][city]", addr.city),
          ("customer_details[address][country]", addr.country),
          ("customer_details[address][line1]", addr.line1),
          ("customer_details[address][line2]", addr.line2),
          ("customer_details[address][postal_code]", addr.postal_code),
          ("customer_details[address][state]", addr.state),
        ]);

        if let Some(ip_address) = customer_details.ip_address {
          body.push(("customer_details[ip_address]", ip_address));
        }
      }
    }

    let response = with_headers(request, config).form(&body).send().await?;

    let tax_calculation = match response.error_for_status() {
      Ok(response) => response.json::<TaxCalculation>().await.unwrap(),
      Err(err) => {
        error!("{:?}", err);

        if err.status() == Some(reqwest::StatusCode::BAD_REQUEST) {
          return Err(TaxCalculationError::BadRequest);
        }

        return Err(err.into());
      }
    };

    Ok(tax_calculation)
  }
}

impl TaxCalculationLineItem {
  pub async fn retrieve(
    config: &Config,
    tax_calculation_id: String,
  ) -> Result<Self, TaxCalculationError> {
    let client = reqwest::Client::new();

    let request = client.get(format!(
      "https://api.stripe.com/v1/tax/calculations/{}/line_items",
      tax_calculation_id
    ));

    let response = with_headers(request, config).send().await?;
    let json = response.json::<TaxCalculationLineItemResponse>().await?;

    Ok(json.data[0])
  }
}
