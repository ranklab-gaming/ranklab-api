use crate::guards::Auth;
use crate::guards::Stripe;
use crate::models::Player;
use crate::response::{MutationResponse, Response};
use rocket::serde::json::Json;
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde;
use serde::{Deserialize, Serialize};

#[derive(Serialize, JsonSchema)]
pub struct CheckoutSession {
  url: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct CreateCheckoutSessionMutation {
  success_url: String,
  cancel_url: String,
}

#[openapi(tag = "Ranklab")]
#[post("/player/stripe-checkout-sessions", data = "<body>")]
pub async fn create(
  auth: Auth<Player>,
  stripe: Stripe,
  body: Json<CreateCheckoutSessionMutation>,
) -> MutationResponse<CheckoutSession> {
  let customer_id = auth
    .0
    .stripe_customer_id
    .unwrap()
    .parse::<stripe::CustomerId>()
    .unwrap();

  let mut params =
    stripe::CreateCheckoutSession::new(body.success_url.as_str(), body.cancel_url.as_str());

  params.customer = Some(customer_id);
  params.mode = Some(stripe::CheckoutSessionMode::Setup);
  params.payment_method_types = Some(Box::new(vec![
    stripe::CreateCheckoutSessionPaymentMethodTypes::Card,
  ]));

  let checkout_session = stripe::CheckoutSession::create(&stripe.0 .0, params)
    .await
    .unwrap();

  Response::success(CheckoutSession {
    url: *checkout_session.url.unwrap(),
  })
}
