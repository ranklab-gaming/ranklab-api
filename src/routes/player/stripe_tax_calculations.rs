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
use uuid::Uuid;

#[derive(Deserialize, JsonSchema)]
pub struct CreateTaxCalculationRequest {
  pub address: Address,
  pub review_id: Uuid,
  pub preview: bool,
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
  let customer_id = player.stripe_customer_id;
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

  let tax_calculation = TaxCalculation::create(
    config,
    CreateTaxCalculation {
      customer: customer_id,
      preview: params.preview,
      price: coach.price.into(),
      reference: match params.preview {
        true => Some("0".to_string()),
        false => None,
      },
      customer_details: CustomerDetails {
        address: Some(params.address.clone()),
        ip_address: None,
      },
    },
  )
  .await
  .map_err(|err| match err {
    TaxCalculationError::BadRequest => MutationError::Status(Status::BadRequest),
    TaxCalculationError::ServerError(err) => MutationError::InternalServerError(err.into()),
  })?;

  if !params.preview {
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
