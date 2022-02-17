use crate::guards::Auth;
use crate::guards::DbConn;
use crate::guards::Stripe;
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
    .map(Into::into)
    .collect();

  Response::success(reviews)
}

#[openapi(tag = "Ranklab")]
#[get("/player/reviews/<id>")]
pub async fn get(id: Uuid, auth: Auth<Player>, db_conn: DbConn) -> QueryResponse<ReviewView> {
  let review = db_conn
    .run(move |conn| {
      use crate::schema::reviews::dsl::{id as review_id, player_id, reviews};
      reviews
        .filter(player_id.eq(auth.0.id).and(review_id.eq(id)))
        .first::<Review>(conn)
    })
    .await?
    .into();

  Response::success(review)
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

  let review = db_conn
    .run(move |conn| {
      use crate::schema::reviews::dsl::*;

      diesel::update(&review)
        .set(stripe_payment_intent_id.eq(payment_intent_id.to_string()))
        .get_result::<Review>(conn)
        .unwrap()
    })
    .await
    .into();

  Response::success(review)
}
