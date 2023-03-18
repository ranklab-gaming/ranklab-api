use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Deserialize, JsonSchema)]
pub struct Address {
  pub city: Option<String>,
  pub country: Option<String>,
  pub line1: Option<String>,
  pub line2: Option<String>,
  pub postal_code: Option<String>,
  pub state: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct TaxCalculation {
  pub amount_total: i64,
  pub tax_amount_exclusive: i64,
  pub tax_amount_inclusive: i64,
  pub id: String,
}

impl TaxCalculation {
  pub fn new(
    amount_total: i64,
    tax_amount_exclusive: i64,
    tax_amount_inclusive: i64,
    id: String,
  ) -> Self {
    let client = reqwest::Client::new();

    let params = [
      ("currency", "usd".to_string()),
      ("customer", player.stripe_customer_id.clone()),
      ("line_items[][price]", coach.price.to_string()),
      ("line_items[][reference]", review.id.to_string()),
      ("preview", "false".to_string()),
    ];

    let response = client
      .post("https://api.stripe.com/v1/tax/calculations")
      .header(
        "Stripe-Version",
        "2022-08-01;tax_calc_beta=v3;tax_txns_beta=v2",
      )
      .header("Authorization", format!("Bearer {}", config.stripe_secret))
      .form(&params)
      .send()
      .await
      .unwrap();

    let tax_calculation = match response.error_for_status() {
      Ok(response) => response.json::<TaxCalculation>().await.unwrap(),
      Err(err) => {
        if err.status() == Some(reqwest::StatusCode::BAD_REQUEST) {
          return Response::mutation_error(Status::UnprocessableEntity);
        }

        return Err(MutationError::InternalServerError(err.into()));
      }
    };
  }

  pub async fn get_tax_calculation(
    &self,
    config: &Config,
    tax_calculation_id: String,
  ) -> anyhow::Result<TaxCalculation> {
    let client = reqwest::Client::new();

    let response = client
      .get(format!(
        "https://api.stripe.com/v1/tax/calculations/{}/{}",
        tax_calculation_id, self.id
      ))
      .header(
        "Stripe-Version",
        "2022-08-01;tax_calc_beta=v3;tax_txns_beta=v2",
      )
      .header("Authorization", format!("Bearer {}", config.stripe_secret))
      .send()
      .await?;

    let tax_calculation = response.json::<TaxCalculation>().await?;

    Ok(tax_calculation)
  }
}
