use crate::config::Config;
use crate::guards::{Auth, DbConn, Jwt, Stripe};
use crate::models::{Coach, Player, Review, ReviewChangeset};
use crate::response::{MutationError, MutationResponse, Response, StatusResponse};
use crate::stripe::TaxCalculation;
use diesel::prelude::*;
use rocket::serde::json::Json;
use rocket::{http::Status, State};
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde::Deserialize;
use std::collections::HashMap;
use stripe::{
  CancelPaymentIntent, CreatePaymentIntent, CreatePaymentIntentTransferData, Currency, CustomerId,
  PaymentIntentCancellationReason, PaymentIntentId, StripeError, UpdateCustomer,
};
use uuid::Uuid;

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
pub struct CreateCheckoutRequest {
  review_id: Uuid,
  billing_address: Address,
}

#[openapi(tag = "Ranklab")]
#[post("/player/checkouts", data = "<checkout>")]
pub async fn create(
  db_conn: DbConn,
  stripe: Stripe,
  auth: Auth<Jwt<Player>>,
  config: &State<Config>,
  checkout: Json<CreateCheckoutRequest>,
) -> MutationResponse<StatusResponse> {
  let player = auth.into_deep_inner();
  let stripe = stripe.into_inner();
  let player_id = player.id;
  let review_id = checkout.review_id;

  let review = db_conn
    .run(move |conn| Review::find_for_player(&review_id, &player_id).first::<Review>(conn))
    .await?;

  let coach_id = review.coach_id;

  let coach = db_conn
    .run(move |conn| Coach::find_by_id(&coach_id).first::<Coach>(conn))
    .await?;

  let customer_id = player.stripe_customer_id.parse::<CustomerId>().unwrap();

  let customer = stripe::Customer::update(
    &stripe,
    &customer_id,
    UpdateCustomer {
      address: Some(stripe::Address {
        city: checkout.billing_address.city.clone(),
        country: checkout.billing_address.country.clone(),
        line1: checkout.billing_address.line1.clone(),
        line2: checkout.billing_address.line2.clone(),
        postal_code: checkout.billing_address.postal_code.clone(),
        state: checkout.billing_address.state.clone(),
      }),
      ..Default::default()
    },
  )
  .await;

  if let Err(err) = customer {
    match &err {
      StripeError::Stripe(_) => return Response::mutation_error(Status::UnprocessableEntity),
      _ => {
        return Err(MutationError::InternalServerError(err.into()));
      }
    }
  }

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

  if let Some(payment_intent_id) = review.stripe_payment_intent_id.clone() {
    stripe::PaymentIntent::cancel(
      &stripe,
      &payment_intent_id.parse::<PaymentIntentId>().unwrap(),
      CancelPaymentIntent {
        cancellation_reason: Some(PaymentIntentCancellationReason::Abandoned),
      },
    )
    .await
    .unwrap();

    let review_id = review.id;

    db_conn
      .run(move |conn| {
        diesel::update(Review::find_for_player(&review_id, &player_id))
          .set(ReviewChangeset::default().stripe_payment_intent_id(None))
          .get_result::<Review>(conn)
          .unwrap()
      })
      .await;
  }

  let mut payment_intent_params = CreatePaymentIntent::new(coach.price.into(), Currency::USD);
  let mut payment_intent_metadata = HashMap::new();

  payment_intent_metadata.insert("tax_calculation_id".to_string(), tax_calculation.id);
  payment_intent_params.metadata = Some(payment_intent_metadata);
  payment_intent_params.customer = Some(customer_id);
  payment_intent_params.application_fee_amount = Some(((coach.price as f32) * 0.2).round() as i64);

  payment_intent_params.transfer_data = Some(CreatePaymentIntentTransferData {
    amount: None,
    destination: coach.stripe_account_id,
  });

  let payment_intent = stripe::PaymentIntent::create(&stripe, payment_intent_params)
    .await
    .unwrap();

  let review = review.clone();

  db_conn
    .run(move |conn| {
      diesel::update(&review)
        .set(
          ReviewChangeset::default().stripe_payment_intent_id(Some(payment_intent.id.to_string())),
        )
        .get_result::<Review>(conn)
        .unwrap()
    })
    .await;

  Response::status(Status::Created)
}
