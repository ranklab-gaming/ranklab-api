use crate::config::Config;
use crate::guards::{Auth, DbConn, Jwt, Stripe};
use crate::models::{Coach, Player, Review};
use crate::response::{MutationError, MutationResponse, Response};
use crate::stripe::{
  Address, CreateTaxCalculation, CustomerDetails, TaxCalculation, TaxCalculationError,
};
use diesel::prelude::*;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use stripe::{CustomerId, UpdateCustomer};
use uuid::Uuid;

#[derive(Deserialize, JsonSchema)]
pub struct CreateTaxCalculationRequest {
  pub address: Option<Address>,
  pub review_id: Uuid,
}

#[derive(Serialize, JsonSchema)]
pub struct CreateTaxCalculationResponse {
  pub tax: i64,
}

#[openapi(tag = "Ranklab")]
#[post("/player/stripe-tax-calculations", data = "<params>")]
pub async fn create(
  auth: Auth<Jwt<Player>>,
  config: &State<Config>,
  db_conn: DbConn,
  stripe: Stripe,
  params: Json<CreateTaxCalculationRequest>,
) -> MutationResponse<CreateTaxCalculationResponse> {
  let player = auth.into_deep_inner();
  let customer_id = player.stripe_customer_id.parse::<CustomerId>().unwrap();
  let review_id = params.review_id;
  let player_id = player.id;
  let stripe = stripe.into_inner();

  let review = db_conn
    .run(move |conn| Review::find_for_player(&review_id, &player_id).first::<Review>(conn))
    .await?;

  let coach_id = review.coach_id;

  let coach = db_conn
    .run(move |conn| Coach::find_by_id(&coach_id).first::<Coach>(conn))
    .await?;

  if let Some(address) = &params.address {
    stripe::Customer::update(
      &stripe,
      &customer_id,
      UpdateCustomer {
        address: Some(stripe::Address {
          city: Some(address.city.clone()),
          country: Some(address.country.clone()),
          line1: Some(address.line1.clone()),
          line2: Some(address.line2.clone()),
          postal_code: Some(address.postal_code.clone()),
          state: Some(address.state.clone()),
        }),
        ..Default::default()
      },
    )
    .await
    .map_err(|err| {
      if let stripe::StripeError::Stripe(err) = &err {
        if err.http_status == 400 {
          return MutationError::Status(Status::BadRequest);
        }
      }

      MutationError::InternalServerError(err.into())
    })?;
  }

  let tax_calculation = TaxCalculation::create(
    config,
    CreateTaxCalculation {
      price: coach.price.into(),
      customer: match &params.address {
        Some(_) => None,
        None => Some(customer_id.to_string()),
      },
      customer_details: params.address.clone().map(|address| CustomerDetails {
        address: Some(address),
        ip_address: None,
      }),
    },
  )
  .await
  .map_err(|err| match err {
    TaxCalculationError::BadRequest => MutationError::Status(Status::BadRequest),
    TaxCalculationError::ServerError(err) => MutationError::InternalServerError(err.into()),
  })?;

  if params.address.is_none() {
    let payment_intent = review.get_payment_intent(&stripe).await;
    let mut metadata = HashMap::new();

    metadata.insert("tax_calculation_id".to_string(), tax_calculation.id);

    stripe::PaymentIntent::update(
      &stripe,
      &payment_intent.id,
      stripe::UpdatePaymentIntent {
        metadata: Some(metadata),
        ..Default::default()
      },
    )
    .await
    .unwrap();
  }

  Response::success(CreateTaxCalculationResponse {
    tax: tax_calculation.tax_amount_exclusive,
  })
}
