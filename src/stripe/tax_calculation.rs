use crate::config::Config;
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Deserialize, JsonSchema)]
pub struct Address {
  pub city: String,
  pub country: String,
  pub line1: String,
  pub line2: String,
  pub postal_code: String,
  pub state: String,
}

impl Default for Address {
  fn default() -> Self {
    Address {
      city: "".to_string(),
      country: "".to_string(),
      line1: "".to_string(),
      line2: "".to_string(),
      postal_code: "".to_string(),
      state: "".to_string(),
    }
  }
}

#[derive(Deserialize, JsonSchema)]
pub struct CustomerDetails {
  pub address: Option<Address>,
  pub ip_address: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct TaxCalculation {
  pub amount_total: i64,
  pub tax_amount_exclusive: i64,
  pub tax_amount_inclusive: i64,
  pub id: String,
}

pub struct CreateTaxCalculation {
  pub customer_details: CustomerDetails,
  pub price: i64,
  pub reference: String,
  pub preview: bool,
}

#[derive(thiserror::Error, Debug)]
pub enum TaxCalculationError {
  #[error("Bad request")]
  BadRequest,
  #[error(transparent)]
  ServerError(#[from] reqwest::Error),
}

impl TaxCalculation {
  pub async fn create(
    config: &Config,
    params: CreateTaxCalculation,
  ) -> Result<Self, TaxCalculationError> {
    let addr = params.customer_details.address.unwrap_or_default();
    let client = reqwest::Client::new();
    let request = client.post("https://api.stripe.com/v1/tax/calculations");
    let ip = params.customer_details.ip_address;

    let params = [
      ("currency", "usd".to_string()),
      ("customer_details[address][city]", addr.city),
      ("customer_details[address][country]", addr.country),
      ("customer_details[address][line1]", addr.line1),
      ("customer_details[address][line2]", addr.line2),
      ("customer_details[address][postal_code]", addr.postal_code),
      ("customer_details[address][state]", addr.state),
      ("customer_details[ip_address]", ip.unwrap_or_default()),
      ("line_items[][price]", params.price.to_string()),
      ("line_items[][reference]", params.reference),
      ("preview", params.preview.to_string()),
    ];

    let response = Self::with_headers(request, config)
      .form(&params)
      .send()
      .await?;

    let tax_calculation = match response.error_for_status() {
      Ok(response) => response.json::<TaxCalculation>().await.unwrap(),
      Err(err) => {
        if err.status() == Some(reqwest::StatusCode::BAD_REQUEST) {
          return Err(TaxCalculationError::BadRequest);
        }

        return Err(err.into());
      }
    };

    Ok(tax_calculation)
  }

  pub async fn retrieve(
    config: &Config,
    tax_calculation_id: String,
  ) -> Result<Self, TaxCalculationError> {
    let client = reqwest::Client::new();

    let request = client.get(format!(
      "https://api.stripe.com/v1/tax/calculations/{}/0",
      tax_calculation_id
    ));

    let response = Self::with_headers(request, config).send().await?;
    let tax_calculation = response.json::<TaxCalculation>().await?;

    Ok(tax_calculation)
  }

  fn with_headers(request: reqwest::RequestBuilder, config: &Config) -> reqwest::RequestBuilder {
    request
      .header(
        "Stripe-Version",
        "2022-08-01;tax_calc_beta=v3;tax_txns_beta=v2",
      )
      .header("Authorization", format!("Bearer {}", config.stripe_secret))
  }
}
