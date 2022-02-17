use crate::guards::Auth;
use crate::guards::DbConn;
use crate::guards::Stripe;
use crate::models::Player;
use crate::models::ReviewIntent;
use crate::response::MutationError;
use crate::response::{MutationResponse, Response};
use crate::views::ReviewIntentView;
use diesel::prelude::*;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde;
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

#[derive(Deserialize, JsonSchema)]
pub struct CreateReviewIntentMutation {
  recording_id: Uuid,
}

#[derive(Deserialize, Validate, JsonSchema)]
pub struct UpdateReviewIntentMutation {
  #[validate(length(min = 1))]
  title: String,
  notes: String,
  #[validate(custom = "crate::games::validate_id")]
  game_id: String,
}

#[openapi(tag = "Ranklab")]
#[post("/player/review-intents", data = "<body>")]
pub async fn create(
  db_conn: DbConn,
  auth: Auth<Player>,
  stripe: Stripe,
  body: Json<CreateReviewIntentMutation>,
) -> MutationResponse<ReviewIntentView> {
  let body_recording_id = body.recording_id.clone();

  let review_intent = db_conn
    .run(move |conn| {
      use crate::schema::review_intents::dsl::*;
      review_intents
        .filter(recording_id.eq(Some(body_recording_id)))
        .first::<ReviewIntent>(conn)
    })
    .await;

  if let Ok(review_intent) = review_intent {
    let payment_intent_id = review_intent
      .stripe_payment_intent_id
      .parse::<stripe::PaymentIntentId>()
      .unwrap();

    let payment_intent = stripe::PaymentIntent::retrieve(&stripe.0 .0, &payment_intent_id, &[])
      .await
      .map_err(|_| MutationError::Status(Status::InternalServerError))?;

    return Response::success(ReviewIntentView::from(review_intent, payment_intent));
  }

  let auth_player_id = auth.0.id.clone();

  let review_intent = db_conn
    .run(move |conn| {
      use crate::schema::review_intents::dsl::*;

      diesel::insert_into(review_intents)
        .values((
          recording_id.eq(Some(body_recording_id)),
          player_id.eq(auth_player_id),
        ))
        .get_result::<ReviewIntent>(conn)
        .unwrap()
    })
    .await;

  let customer_id = auth
    .0
    .stripe_customer_id
    .unwrap()
    .parse::<stripe::CustomerId>()
    .unwrap();

  let mut params = stripe::CreatePaymentIntent::new(10_00, stripe::Currency::USD);

  params.customer = Some(customer_id);
  params.description = Some("Recording payment");
  params.automatic_payment_methods =
    Some(stripe::CreatePaymentIntentAutomaticPaymentMethods { enabled: true }.into());
  params.setup_future_usage = Some(stripe::PaymentIntentSetupFutureUsage::OnSession);

  let payment_intent = stripe::PaymentIntent::create(&stripe.0 .0, params)
    .await
    .unwrap();

  let payment_intent_id = payment_intent.id.clone();

  let review_intent = db_conn
    .run(move |conn| {
      use crate::schema::review_intents::dsl::*;

      diesel::update(&review_intent)
        .set(stripe_payment_intent_id.eq(payment_intent_id.to_string()))
        .get_result::<ReviewIntent>(conn)
        .unwrap()
    })
    .await;

  Response::success(ReviewIntentView::from(review_intent, payment_intent))
}

#[openapi(tag = "Ranklab")]
#[put("/player/stripe-payment-intents/<recording_id>", data = "<body>")]
pub async fn update(
  db_conn: DbConn,
  auth: Auth<Player>,
  stripe: Stripe,
  recording_id: Uuid,
  body: Json<UpdateReviewIntentMutation>,
) -> MutationResponse<ReviewIntentView> {
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

  let review_intent = db_conn
    .run(move |conn| {
      use crate::schema::review_intents::dsl::{
        game_id, notes, recording_id as recording_id_column, title,
      };

      diesel::update(
        crate::schema::review_intents::table.filter(recording_id_column.eq(Some(recording_id))),
      )
      .set((
        title.eq(body.title.clone()),
        notes.eq(body.notes.clone()),
        game_id.eq(body.game_id.clone()),
      ))
      .get_result::<ReviewIntent>(conn)
      .unwrap()
    })
    .await;

  let payment_intent_id = review_intent
    .stripe_payment_intent_id
    .parse::<stripe::PaymentIntentId>()
    .unwrap();

  let payment_intent = stripe::PaymentIntent::retrieve(&stripe.0 .0, &payment_intent_id, &[])
    .await
    .map_err(|_| MutationError::Status(Status::InternalServerError))?;

  Response::success(ReviewIntentView::from(review_intent, payment_intent))
}
