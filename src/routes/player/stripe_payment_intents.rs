use std::collections::HashMap;

use crate::guards::Auth;
use crate::guards::DbConn;
use crate::guards::Stripe;
use crate::models::Player;
use crate::models::Recording;
use crate::response::MutationError;
use crate::response::{MutationResponse, Response};
use diesel::prelude::*;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Serialize, JsonSchema)]
pub struct PaymentIntent {
  client_secret: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct CreatePaymentIntentMutation {
  recording_id: Uuid,
}

#[derive(Deserialize, Validate, JsonSchema)]
pub struct UpdatePaymentIntentMutation {
  #[validate(length(min = 1))]
  title: String,
  notes: String,
  #[validate(custom = "crate::games::validate_id")]
  game_id: String,
}

#[openapi(tag = "Ranklab")]
#[post("/player/stripe-payment-intents", data = "<body>")]
pub async fn create(
  db_conn: DbConn,
  auth: Auth<Player>,
  stripe: Stripe,
  body: Json<CreatePaymentIntentMutation>,
) -> MutationResponse<PaymentIntent> {
  let recording_id = body.recording_id.clone();

  let recording = db_conn
    .run(move |conn| {
      use crate::schema::recordings::dsl::*;
      recordings
        .filter(id.eq(recording_id))
        .first::<Recording>(conn)
    })
    .await?;

  if let Some(payment_intent_id) = recording.stripe_payment_intent_id {
    let payment_intent_id = payment_intent_id
      .parse::<stripe::PaymentIntentId>()
      .unwrap();

    let payment_intent = stripe::PaymentIntent::retrieve(&stripe.0 .0, &payment_intent_id, &[])
      .await
      .map_err(|_| MutationError::Status(Status::InternalServerError))?;

    return Response::success(PaymentIntent {
      client_secret: *payment_intent.client_secret.unwrap(),
    });
  }

  let customer_id = auth
    .0
    .stripe_customer_id
    .unwrap()
    .parse::<stripe::CustomerId>()
    .unwrap();

  let mut params = stripe::CreatePaymentIntent::new(10_00, stripe::Currency::USD);

  params.customer = Some(customer_id);
  params.description = Some("Recording payment");
  params.payment_method_types = Some(vec!["card".to_string()].into());
  params.setup_future_usage = Some(stripe::PaymentIntentSetupFutureUsage::OnSession);

  let payment_intent = stripe::PaymentIntent::create(&stripe.0 .0, params)
    .await
    .unwrap();

  let payment_intent_id = payment_intent.id.clone();

  db_conn
    .run(move |conn| {
      use crate::schema::recordings::dsl::*;

      diesel::update(crate::schema::recordings::table.find(recording.id))
        .set(stripe_payment_intent_id.eq(payment_intent_id.to_string()))
        .get_result::<Recording>(conn)
        .unwrap()
    })
    .await;

  Response::success(PaymentIntent {
    client_secret: *payment_intent.client_secret.unwrap(),
  })
}

#[openapi(tag = "Ranklab")]
#[put("/player/stripe-payment-intents/<recording_id>", data = "<body>")]
pub async fn update(
  db_conn: DbConn,
  auth: Auth<Player>,
  stripe: Stripe,
  recording_id: Uuid,
  body: Json<UpdatePaymentIntentMutation>,
) -> MutationResponse<PaymentIntent> {
  if let Err(errors) = body.validate() {
    return Response::validation_error(errors);
  }

  let game = auth
    .0
    .games
    .clone()
    .into_iter()
    .find(|g| g.game_id == body.game_id);

  if game.is_none() {
    return Response::mutation_error(Status::BadRequest);
  }

  let recording = db_conn
    .run(move |conn| {
      use crate::schema::recordings::dsl::*;
      recordings
        .filter(id.eq(recording_id).and(player_id.eq(auth.0.id)))
        .first::<Recording>(conn)
    })
    .await?;

  if recording.stripe_payment_intent_id.is_none() {
    return Response::mutation_error(Status::BadRequest);
  }

  let payment_intent_id = recording
    .stripe_payment_intent_id
    .unwrap()
    .parse::<stripe::PaymentIntentId>()
    .unwrap();

  let mut params = stripe::UpdatePaymentIntent::new();

  params.metadata = Some(HashMap::from([
    ("title".to_string(), body.title.clone()),
    ("notes".to_string(), body.notes.clone()),
    ("game_id".to_string(), body.game_id.clone()),
  ]));

  let payment_intent = stripe::PaymentIntent::update(&stripe.0 .0, &payment_intent_id, params)
    .await
    .map_err(|_| MutationError::Status(Status::BadRequest))?;

  Response::success(PaymentIntent {
    client_secret: *payment_intent.client_secret.unwrap(),
  })
}
