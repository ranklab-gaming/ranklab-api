use crate::guards::{Auth, Stripe};
use crate::models::Player;
use crate::response::{QueryResponse, Response};
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde::Serialize;

#[derive(Serialize, JsonSchema)]
pub struct PaymentMethod {
  id: String,
  brand: String,
  last4: String,
}

#[openapi(tag = "Ranklab")]
#[get("/player/stripe-payment-methods")]
pub async fn list(auth: Auth<Player>, stripe: Stripe) -> QueryResponse<Vec<PaymentMethod>> {
  let mut payment_method_params = stripe::ListPaymentMethods::new();

  payment_method_params.type_ = Some(stripe::PaymentMethodTypeFilter::Card);

  payment_method_params.customer = Some(
    auth
      .0
      .stripe_customer_id
      .parse::<stripe::CustomerId>()
      .unwrap(),
  );

  let payment_methods = stripe::PaymentMethod::list(&stripe.0 .0, &payment_method_params)
    .await
    .unwrap()
    .data
    .into_iter()
    .map(|payment_method| PaymentMethod {
      id: payment_method.id.to_string(),
      brand: payment_method.card.as_ref().unwrap().brand.to_string(),
      last4: payment_method.card.as_ref().unwrap().last4.to_string(),
    })
    .collect();

  Response::success(payment_methods)
}
