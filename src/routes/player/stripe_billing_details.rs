use crate::guards::{Auth, Jwt, Stripe};
use crate::models::Player;
use crate::response::{MutationError, MutationResponse, Response, StatusResponse};
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde::Deserialize;
use stripe::CustomerId;

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
pub struct UpdateBillingDetailsRequest {
  pub address: Address,
}

#[openapi(tag = "Ranklab")]
#[put("/player/stripe-billing-details", data = "<body>")]
pub async fn update(
  auth: Auth<Jwt<Player>>,
  stripe: Stripe,
  body: Json<UpdateBillingDetailsRequest>,
) -> MutationResponse<StatusResponse> {
  let player = auth.into_deep_inner();
  let stripe = stripe.into_inner();

  stripe::Customer::update(
    &stripe,
    &player.stripe_customer_id.parse::<CustomerId>().unwrap(),
    stripe::UpdateCustomer {
      address: Some(stripe::Address {
        city: body.address.city.clone(),
        country: body.address.country.clone(),
        line1: body.address.line1.clone(),
        line2: body.address.line2.clone(),
        postal_code: body.address.postal_code.clone(),
        state: body.address.state.clone(),
      }),

      ..Default::default()
    },
  )
  .await
  .map_err(|err| {
    if let stripe::StripeError::Stripe(err) = &err {
      if err.http_status == 400 {
        return MutationError::Status(Status::UnprocessableEntity);
      }
    }

    MutationError::InternalServerError(err.into())
  })?;

  Response::status(Status::Ok)
}
