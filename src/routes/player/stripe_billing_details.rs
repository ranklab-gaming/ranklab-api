use crate::guards::{Auth, Jwt, Stripe};
use crate::models::Player;
use crate::response::{MutationError, MutationResponse, QueryResponse, Response, StatusResponse};
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use stripe::CustomerId;

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct Address {
  pub city: Option<String>,
  pub country: Option<String>,
  pub line_1: Option<String>,
  pub line_2: Option<String>,
  pub postal_code: Option<String>,
  pub state: Option<String>,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone)]
pub struct BillingDetails {
  pub address: Option<Address>,
  pub name: Option<String>,
  pub phone: Option<String>,
}

#[openapi(tag = "Ranklab")]
#[get("/player/stripe-billing-details")]
pub async fn get(auth: Auth<Jwt<Player>>, stripe: Stripe) -> QueryResponse<BillingDetails> {
  let player = auth.into_deep_inner();
  let stripe = stripe.into_inner();

  let customer = stripe::Customer::retrieve(
    &stripe,
    &player.stripe_customer_id.parse::<CustomerId>().unwrap(),
    Default::default(),
  )
  .await
  .unwrap();

  Response::success(BillingDetails {
    address: customer.address.map(|address| Address {
      city: address.city,
      country: address.country,
      line_1: address.line1,
      line_2: address.line2,
      postal_code: address.postal_code,
      state: address.state,
    }),
    name: customer.name,
    phone: customer.phone,
  })
}

#[openapi(tag = "Ranklab")]
#[put("/player/stripe-billing-details", data = "<body>")]
pub async fn update(
  auth: Auth<Jwt<Player>>,
  stripe: Stripe,
  body: Json<BillingDetails>,
) -> MutationResponse<StatusResponse> {
  let player = auth.into_deep_inner();
  let stripe = stripe.into_inner();

  stripe::Customer::update(
    &stripe,
    &player.stripe_customer_id.parse::<CustomerId>().unwrap(),
    stripe::UpdateCustomer {
      address: body.address.clone().map(|address| stripe::Address {
        city: address.city,
        country: address.country,
        line1: address.line_1,
        line2: address.line_2,
        postal_code: address.postal_code,
        state: address.state,
      }),
      name: body.name.as_deref(),
      phone: body.phone.as_deref(),
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
