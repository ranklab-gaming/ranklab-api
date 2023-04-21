use crate::config::Config;
use crate::data_types::{RecordingState, ReviewState};
use crate::guards::{Auth, DbConn, Jwt, Stripe};
use crate::models::{Coach, Player, Recording, Review, ReviewChangeset};
use crate::pagination::{Paginate, PaginatedResult};
use crate::response::{MutationError, MutationResponse, QueryResponse, Response, StatusResponse};
use crate::schema::{coaches, reviews};
use crate::stripe::{RequestError, TaxCalculation};
use crate::views::{ReviewView, ReviewViewOptions};
use diesel::prelude::*;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde;
use serde::Deserialize;
use std::collections::HashMap;
use stripe::{
  CancelPaymentIntent, CreatePaymentIntent, CreateRefund, Currency, Expandable,
  PaymentIntentCancellationReason, PaymentIntentId,
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

  let records = paginated_reviews.records.clone();

  let coaches = db_conn
    .run(move |conn| {
      Coach::filter_by_ids(records.into_iter().map(|review| review.coach_id).collect())
        .load::<Coach>(conn)
    })
    .await?;

  let records = paginated_reviews.records.clone();

  let recordings = db_conn
    .run(move |conn| {
      Recording::filter_by_ids(
        records
          .into_iter()
          .map(|review| review.recording_id)
          .collect(),
      )
      .load::<Recording>(conn)
    })
    .await?;

  let review_views = paginated_reviews
    .records
    .clone()
    .into_iter()
    .map(|review| {
      let coach_id = review.coach_id;
      let recording_id = review.recording_id;

      ReviewView::new(
        review,
        ReviewViewOptions {
          payment_intent: None,
          tax_calculation: None,
          coach: coaches.iter().find(|coach| coach.id == coach_id).cloned(),
          player: None,
          recording: recordings
            .iter()
            .find(|recording| recording.id == recording_id)
            .cloned(),
        },
      )
    })
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
  config: &State<Config>,
) -> QueryResponse<ReviewView> {
  let review = db_conn
    .run(move |conn| Review::find_for_player(&id, &auth.into_deep_inner().id).first::<Review>(conn))
    .await?;

  let coach_id = review.coach_id;
  let recording_id = review.recording_id;
  let stripe = stripe.into_inner();

  let coach = db_conn
    .run(move |conn| Coach::find_by_id(&coach_id).first::<Coach>(conn))
    .await?;

  let recording = db_conn
    .run(move |conn| Recording::find_by_id(&recording_id).first::<Recording>(conn))
    .await?;

  if review.state == ReviewState::AwaitingPayment {
    let payment_intent = review.get_payment_intent(&stripe).await;
    let tax_calculation_id = &payment_intent.metadata["tax_calculation_id"];

    let tax_calculation = TaxCalculation::retrieve(config, tax_calculation_id.to_string())
      .await
      .unwrap();

    Response::success(ReviewView::new(
      review,
      ReviewViewOptions {
        payment_intent: Some(payment_intent),
        tax_calculation: Some(tax_calculation),
        coach: Some(coach),
        player: None,
        recording: Some(recording),
      },
    ))
  } else {
    Response::success(ReviewView::new(
      review,
      ReviewViewOptions {
        payment_intent: None,
        tax_calculation: None,
        coach: Some(coach),
        player: None,
        recording: Some(recording),
      },
    ))
  }
}

#[derive(Deserialize, JsonSchema)]
pub struct CreateReviewRequest {
  recording_id: Uuid,
  notes: String,
  coach_id: Uuid,
}

#[openapi(tag = "Ranklab")]
#[post("/player/reviews", data = "<body>")]
pub async fn create(
  db_conn: DbConn,
  auth: Auth<Jwt<Player>>,
  body: Json<CreateReviewRequest>,
  stripe: Stripe,
  config: &State<Config>,
) -> MutationResponse<ReviewView> {
  let recording_id = body.recording_id;
  let player = auth.into_deep_inner();
  let player_id = player.id;
  let coach_id = body.coach_id;
  let game_id = player.game_id.clone();
  let stripe = stripe.into_inner();

  let coach = db_conn
    .run(move |conn| Coach::find_for_game_id(&coach_id, &game_id).first::<Coach>(conn))
    .await?;

  let recording = db_conn
    .run(move |conn| Recording::find_by_id(&recording_id).first::<Recording>(conn))
    .await?;

  if recording.state == RecordingState::Created {
    return Response::mutation_error(Status::UnprocessableEntity);
  }

  let customer_id = player
    .stripe_customer_id
    .parse::<stripe::CustomerId>()
    .unwrap();

  let tax_calculation = TaxCalculation::create(config, &customer_id, coach.price.into())
    .await
    .map_err(|err| match err {
      RequestError::BadRequest(_) => MutationError::Status(Status::UnprocessableEntity),
      RequestError::ServerError(err) => MutationError::InternalServerError(err.into()),
    })?;

  let mut payment_intent_params =
    CreatePaymentIntent::new(tax_calculation.amount_total, Currency::USD);

  let mut payment_intent_metadata = HashMap::new();

  payment_intent_metadata.insert("tax_calculation_id".to_string(), tax_calculation.id);
  payment_intent_params.customer = Some(customer_id);
  payment_intent_params.metadata = Some(payment_intent_metadata);

  let payment_intent = stripe::PaymentIntent::create(&stripe, payment_intent_params)
    .await
    .unwrap();

  let review = db_conn
    .run(move |conn| {
      diesel::insert_into(reviews::table)
        .values(
          ReviewChangeset::default()
            .recording_id(recording_id)
            .player_id(player_id)
            .notes(ammonia::clean(&body.notes))
            .coach_id(coach.id)
            .stripe_payment_intent_id(payment_intent.id.to_string()),
        )
        .get_result::<Review>(conn)
        .unwrap()
    })
    .await;

  Response::success(review.into())
}

#[derive(Deserialize, JsonSchema)]
#[schemars(rename = "PlayerUpdateReviewRequest")]
pub struct UpdateReviewRequest {
  accepted: Option<bool>,
  cancelled: Option<bool>,
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

  let client = stripe
    .into_inner()
    .with_strategy(stripe::RequestStrategy::Idempotent(id.to_string()));

  let existing_review: Review = db_conn
    .run(move |conn| Review::find_for_player(&id, &auth_id).first(conn))
    .await?;

  let payment_intent = existing_review.get_payment_intent(&client).await;

  if let Some(accepted) = review.accepted {
    if !accepted || existing_review.state == ReviewState::Accepted {
      return Response::success(existing_review.into());
    }

    if existing_review.state != ReviewState::Published {
      return Response::mutation_error(Status::UnprocessableEntity);
    }

    let review_coach_id = existing_review.coach_id;

    let coach: Coach = db_conn
      .run(move |conn| coaches::table.find(&review_coach_id).first(conn).unwrap())
      .await;

    let mut transfer_params =
      stripe::CreateTransfer::new(stripe::Currency::USD, coach.stripe_account_id);

    transfer_params.amount = Some((payment_intent.amount as f64 * 0.8) as i64);

    let charge_id = match payment_intent.latest_charge {
      Some(Expandable::Id(charge_id)) => charge_id,
      Some(Expandable::Object(charge)) => charge.id,
      None => panic!("No charge found"),
    };

    transfer_params.source_transaction = Some(charge_id);

    stripe::Transfer::create(&client, transfer_params)
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

    let coach_id = updated_review.coach_id;
    let recording_id = updated_review.recording_id;

    let coach = db_conn
      .run(move |conn| Coach::find_by_id(&coach_id).first::<Coach>(conn))
      .await?;

    let recording = db_conn
      .run(move |conn| Recording::find_by_id(&recording_id).first::<Recording>(conn))
      .await?;

    return Response::success(ReviewView::new(
      updated_review,
      ReviewViewOptions {
        payment_intent: None,
        tax_calculation: None,
        coach: Some(coach),
        player: None,
        recording: Some(recording),
      },
    ));
  }

  if let Some(cancelled) = review.cancelled {
    if !cancelled || existing_review.state == ReviewState::Refunded {
      return Response::success(existing_review.into());
    }

    if existing_review.state != ReviewState::AwaitingReview {
      return Response::mutation_error(Status::UnprocessableEntity);
    }

    let mut create_refund = CreateRefund::new();

    create_refund.payment_intent = Some(payment_intent.id.clone());

    stripe::Refund::create(&client, create_refund)
      .await
      .unwrap();

    let updated_review = db_conn
      .run(move |conn| {
        diesel::update(&existing_review)
          .set(ReviewChangeset::default().state(ReviewState::Refunded))
          .get_result::<Review>(conn)
          .unwrap()
      })
      .await;

    let coach_id = updated_review.coach_id;
    let recording_id = updated_review.recording_id;

    let coach = db_conn
      .run(move |conn| Coach::find_by_id(&coach_id).first::<Coach>(conn))
      .await?;

    let recording = db_conn
      .run(move |conn| Recording::find_by_id(&recording_id).first::<Recording>(conn))
      .await?;

    return Response::success(ReviewView::new(
      updated_review,
      ReviewViewOptions {
        payment_intent: None,
        tax_calculation: None,
        coach: Some(coach),
        recording: Some(recording),
        player: None,
      },
    ));
  }

  Response::success(existing_review.into())
}

#[openapi(tag = "Ranklab")]
#[delete("/player/reviews/<id>")]
pub async fn delete(
  id: Uuid,
  auth: Auth<Jwt<Player>>,
  db_conn: DbConn,
  stripe: Stripe,
) -> MutationResponse<StatusResponse> {
  let auth_id = auth.into_deep_inner().id;
  let client = stripe.into_inner();

  let existing_review: Review = db_conn
    .run(move |conn| Review::find_for_player(&id, &auth_id).first(conn))
    .await?;

  if existing_review.state != ReviewState::AwaitingPayment {
    return Response::mutation_error(Status::UnprocessableEntity);
  }

  stripe::PaymentIntent::cancel(
    &client,
    &existing_review
      .stripe_payment_intent_id
      .parse::<PaymentIntentId>()
      .unwrap(),
    CancelPaymentIntent {
      cancellation_reason: Some(PaymentIntentCancellationReason::RequestedByCustomer),
    },
  )
  .await
  .unwrap();

  db_conn
    .run(move |conn| diesel::delete(&existing_review).execute(conn).unwrap())
    .await;

  Response::status(Status::NoContent)
}
