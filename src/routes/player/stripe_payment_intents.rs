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

#[derive(Serialize, JsonSchema)]
pub struct PaymentIntent {
  client_secret: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct CreatePaymentIntentMutation {
  recording_id: Uuid,
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
  params.capture_method = Some(stripe::PaymentIntentCaptureMethod::Manual);
  params.setup_future_usage = Some(stripe::PaymentIntentSetupFutureUsage::OnSession);

  let payment_intent = stripe::PaymentIntent::create(&stripe.0 .0, params)
    .await
    .unwrap();

  Response::success(PaymentIntent {
    client_secret: *payment_intent.client_secret.unwrap(),
  })
}
