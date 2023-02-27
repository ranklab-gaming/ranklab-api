use rocket_okapi::openapi;
use serde::Deserialize;

use crate::guards::Stripe;
use crate::response::{MutationResponse, Response};

#[derive(Deserialize)]
struct CountrySpec {
  supported_transfer_countries: Vec<String>,
}

#[openapi(tag = "Ranklab")]
#[post("/coach/stripe-country-specs")]
pub async fn list(stripe: Stripe) -> MutationResponse<Vec<String>> {
  let country_spec = &stripe
    .into_inner()
    .get::<CountrySpec>(&format!("/country_specs/{}", "US"))
    .await
    .unwrap();

  Response::success(country_spec.supported_transfer_countries.clone())
}
