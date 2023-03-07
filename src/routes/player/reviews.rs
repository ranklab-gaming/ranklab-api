use crate::data_types::ReviewState;
use crate::guards::{Auth, DbConn, Jwt, Stripe};
use crate::models::{Coach, Player, Review, ReviewChangeset};
use crate::pagination::{Paginate, PaginatedResult};
use crate::response::{MutationResponse, QueryResponse, Response};
use crate::schema::{coaches, reviews};
use crate::views::ReviewView;
use diesel::prelude::*;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde;
use serde::Deserialize;
use stripe::{
  CreatePaymentIntent, CreatePaymentIntentTransferData, Currency, Expandable, PaymentIntentId,
};
use uuid::Uuid;

#[derive(FromForm, JsonSchema)]
pub struct ListReviewsQuery {
  page: Option<i64>,
  archived: Option<bool>,
}

#[openapi(tag = "Ranklab")]
#[get("/player/reviews?<params..>")]
pub async fn list(
  params: ListReviewsQuery,
  auth: Auth<Jwt<Player>>,
  db_conn: DbConn,
) -> QueryResponse<PaginatedResult<ReviewView>> {
  let paginated_reviews: PaginatedResult<Review> = db_conn
    .run(move |conn| {
      Review::filter_for_player(&auth.into_deep_inner().id, params.archived.unwrap_or(false))
        .paginate(params.page.unwrap_or(1))
        .load_and_count_pages::<Review>(conn)
        .unwrap()
    })
    .await;

  let review_views = paginated_reviews
    .records
    .clone()
    .into_iter()
    .map(|review| ReviewView::from(review, None))
    .collect();

  Response::success(paginated_reviews.records(review_views))
}

#[openapi(tag = "Ranklab")]
#[get("/player/reviews/<id>")]
pub async fn get(
  id: Uuid,
  auth: Auth<Jwt<Player>>,
  db_conn: DbConn,
  stripe: Stripe,
) -> QueryResponse<ReviewView> {
  let review = db_conn
    .run(move |conn| Review::find_for_player(&id, &auth.into_deep_inner().id).first::<Review>(conn))
    .await?;

  let payment_intent = review.get_payment_intent(&stripe.into_inner()).await;

  Response::success(ReviewView::from(review, Some(payment_intent)))
}

#[derive(Deserialize, JsonSchema)]
pub struct CreateReviewRequest {
  recording_id: Uuid,
  #[validate(length(min = 1))]
  title: String,
  notes: String,
  #[validate(custom = "crate::games::validate_id")]
  game_id: String,
  coach_id: Uuid,
}

#[openapi(tag = "Ranklab")]
#[post("/player/reviews", data = "<body>")]
pub async fn create(
  db_conn: DbConn,
  auth: Auth<Jwt<Player>>,
  stripe: Stripe,
  body: Json<CreateReviewRequest>,
) -> MutationResponse<ReviewView> {
  let body_recording_id = body.recording_id;
  let player = auth.into_deep_inner();
  let auth_player_id = player.id;
  let coach_id = body.coach_id;

  let coach = db_conn
    .run(move |conn| coaches::table.find(coach_id).first::<Coach>(conn))
    .await?;

  if coach.game_id != body.game_id {
    return Response::mutation_error(Status::UnprocessableEntity);
  }

  let customer_id = player
    .stripe_customer_id
    .parse::<stripe::CustomerId>()
    .unwrap();

  let mut payment_intent_params = CreatePaymentIntent::new(coach.price.into(), Currency::USD);

  payment_intent_params.customer = Some(customer_id);
  payment_intent_params.application_fee_amount = Some(((coach.price as f32) * 0.2).round() as i64);

  payment_intent_params.transfer_data = Some(CreatePaymentIntentTransferData {
    amount: None,
    destination: coach.stripe_account_id,
  });

  let payment_intent = stripe::PaymentIntent::create(&stripe.into_inner(), payment_intent_params)
    .await
    .unwrap();

  let review = db_conn
    .run(move |conn| {
      diesel::insert_into(reviews::table)
        .values(
          ReviewChangeset::default()
            .recording_id(body_recording_id)
            .player_id(auth_player_id)
            .title(body.title.clone())
            .notes(ammonia::clean(&body.notes))
            .game_id(body.game_id.clone())
            .coach_id(body.coach_id)
            .stripe_payment_intent_id(payment_intent.id.to_string()),
        )
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
  auth: Auth<Jwt<Player>>,
  db_conn: DbConn,
  stripe: Stripe,
) -> MutationResponse<ReviewView> {
  let auth_id = auth.into_deep_inner().id;

  let existing_review: Review = db_conn
    .run(move |conn| Review::find_for_player(&id, &auth_id).first(conn))
    .await?;

  if !review.accepted {
    return Response::success(ReviewView::from(existing_review, None));
  }

  let review_coach_id = existing_review.coach_id;
  let stripe = stripe.into_inner();

  let coach: Coach = db_conn
    .run(move |conn| coaches::table.find(&review_coach_id).first(conn).unwrap())
    .await;

  let stripe_payment_intent_id = existing_review
    .stripe_payment_intent_id
    .parse::<PaymentIntentId>()
    .unwrap();

  let payment_intent = stripe::PaymentIntent::retrieve(&stripe, &stripe_payment_intent_id, &[])
    .await
    .unwrap();

  let mut transfer_params =
    stripe::CreateTransfer::new(stripe::Currency::USD, coach.stripe_account_id);

  transfer_params.amount = Some((payment_intent.amount as f64 * 0.8) as i64);

  let charge_id = match payment_intent.latest_charge {
    Some(Expandable::Id(charge_id)) => charge_id,
    Some(Expandable::Object(charge)) => charge.id,
    None => panic!("No charge found"),
  };

  transfer_params.source_transaction = Some(charge_id);

  stripe::Transfer::create(&stripe, transfer_params)
    .await
    .unwrap();

  let updated_review = db_conn
    .run(move |conn| {
      diesel::update(&existing_review)
        .set(ReviewChangeset::default().state(ReviewState::Accepted))
        .get_result::<Review>(conn)
        .unwrap()
    })
    .await;

  Response::success(ReviewView::from(updated_review, None))
}
