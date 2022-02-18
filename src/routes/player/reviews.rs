use crate::data_types::ReviewState;
use crate::guards::Auth;
use crate::guards::DbConn;
use crate::guards::Stripe;
use crate::models::Coach;
use crate::models::Player;
use crate::models::Review;
use crate::response::QueryResponse;
use crate::response::{MutationResponse, Response};
use crate::views::ReviewView;
use diesel::prelude::*;
use rocket::serde::json::Json;
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde;
use serde::Deserialize;
use uuid::Uuid;

#[openapi(tag = "Ranklab")]
#[get("/player/reviews")]
pub async fn list(auth: Auth<Player>, db_conn: DbConn) -> QueryResponse<Vec<ReviewView>> {
  let reviews: Vec<ReviewView> = db_conn
    .run(move |conn| {
      use crate::schema::reviews::dsl::*;
      reviews
        .filter(player_id.eq(auth.0.id))
        .load::<Review>(conn)
        .unwrap()
    })
    .await
    .into_iter()
    .map(|review| ReviewView::from(review, None))
    .collect();

  Response::success(reviews)
}

#[openapi(tag = "Ranklab")]
#[get("/player/reviews/<id>")]
pub async fn get(
  id: Uuid,
  auth: Auth<Player>,
  db_conn: DbConn,
  stripe: Stripe,
) -> QueryResponse<ReviewView> {
  let review = db_conn
    .run(move |conn| {
      use crate::schema::reviews::dsl::{id as review_id, player_id, reviews};
      reviews
        .filter(player_id.eq(auth.0.id).and(review_id.eq(id)))
        .first::<Review>(conn)
    })
    .await?;

  let stripe_payment_intent_id = review
    .stripe_payment_intent_id
    .parse::<stripe::PaymentIntentId>()
    .unwrap();

  let payment_intent =
    stripe::PaymentIntent::retrieve(&stripe.0 .0, &stripe_payment_intent_id, &[])
      .await
      .unwrap();

  Response::success(ReviewView::from(review, Some(payment_intent)))
}

#[derive(Deserialize, JsonSchema)]
pub struct CreateReviewMutation {
  recording_id: Uuid,
  #[validate(length(min = 1))]
  title: String,
  notes: String,
  #[validate(custom = "crate::games::validate_id")]
  game_id: String,
}

#[openapi(tag = "Ranklab")]
#[post("/player/reviews", data = "<body>")]
pub async fn create(
  db_conn: DbConn,
  auth: Auth<Player>,
  stripe: Stripe,
  body: Json<CreateReviewMutation>,
) -> MutationResponse<ReviewView> {
  let body_recording_id = body.recording_id.clone();
  let auth_player_id = auth.0.id.clone();

  let review = db_conn
    .run(move |conn| {
      use crate::schema::reviews::dsl::*;

      diesel::insert_into(reviews)
        .values((
          recording_id.eq(body_recording_id),
          player_id.eq(auth_player_id),
          title.eq(body.title.clone()),
          notes.eq(body.notes.clone()),
          game_id.eq(body.game_id.clone()),
        ))
        .get_result::<Review>(conn)
        .unwrap()
    })
    .await;

  let customer_id = auth
    .0
    .stripe_customer_id
    .unwrap()
    .parse::<stripe::CustomerId>()
    .unwrap();

  let mut params = stripe::CreatePaymentIntent::new(10_00, stripe::Currency::DKK);

  params.customer = Some(customer_id);
  params.description = Some("Recording payment");
  params.payment_method_types = Some(vec!["card".to_string()].into());
  params.setup_future_usage = Some(stripe::PaymentIntentSetupFutureUsage::OnSession);

  let payment_intent = stripe::PaymentIntent::create(&stripe.0 .0, params)
    .await
    .unwrap();

  let payment_intent_id = payment_intent.id.clone();

  let review = db_conn
    .run(move |conn| {
      use crate::schema::reviews::dsl::*;

      diesel::update(&review)
        .set(stripe_payment_intent_id.eq(payment_intent_id.to_string()))
        .get_result::<Review>(conn)
        .unwrap()
    })
    .await;

  Response::success(ReviewView::from(review, None))
}

#[derive(Deserialize, JsonSchema)]
#[schemars(rename = "PlayerUpdateReviewRequest")]
pub struct UpdateReviewRequest {
  accepted: bool,
}

#[openapi(tag = "Ranklab")]
#[put("/player/reviews/<id>", data = "<review>")]
pub async fn update(
  id: Uuid,
  review: Json<UpdateReviewRequest>,
  auth: Auth<Player>,
  db_conn: DbConn,
  stripe: Stripe,
) -> MutationResponse<ReviewView> {
  let auth_id = auth.0.id.clone();

  let existing_review = db_conn
    .run(move |conn| {
      use crate::schema::reviews::dsl::{id as review_id, player_id, reviews, state};

      reviews
        .filter(
          review_id
            .eq(id)
            .and(state.eq(ReviewState::Published).or(player_id.eq(auth_id))),
        )
        .first::<Review>(conn)
    })
    .await?;

  if !review.accepted {
    return Response::success(ReviewView::from(existing_review, None));
  }

  let updated_review = db_conn
    .run(move |conn| {
      use crate::schema::reviews::dsl::state;

      diesel::update(&existing_review)
        .set(state.eq(ReviewState::Accepted))
        .get_result::<Review>(conn)
        .unwrap()
    })
    .await;

  let review_coach_id = updated_review.coach_id.unwrap().clone();

  let coach = db_conn
    .run(move |conn| {
      use crate::schema::coaches::dsl::{coaches, id as coach_id};

      coaches
        .filter(coach_id.eq(review_coach_id))
        .first::<Coach>(conn)
        .unwrap()
    })
    .await;

  let stripe_payment_intent_id = updated_review
    .stripe_payment_intent_id
    .parse::<stripe::PaymentIntentId>()
    .unwrap();

  let payment_intent =
    stripe::PaymentIntent::retrieve(&stripe.0 .0, &stripe_payment_intent_id, &[])
      .await
      .unwrap();

  let mut transfer_params =
    stripe::CreateTransfer::new(stripe::Currency::USD, coach.stripe_account_id.unwrap());
  transfer_params.amount = Some((payment_intent.amount as f64 * 0.8) as i64);
  transfer_params.source_transaction = Some(payment_intent.charges.data[0].id.clone());

  stripe::Transfer::create(&stripe.0 .0, transfer_params)
    .await
    .unwrap();

  Response::success(ReviewView::from(updated_review, None))
}
