use crate::data_types::ReviewState;
use crate::guards::Auth;
use crate::guards::DbConn;
use crate::guards::Stripe;
use crate::models::{Coach, Player, Review};
use crate::response::{MutationResponse, QueryResponse, Response};
use crate::views::ReviewView;
use diesel::prelude::*;
use rocket::serde::json::Json;
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

#[derive(FromForm, JsonSchema)]
pub struct ListReviewsQuery {
  pending: Option<bool>,
}

#[openapi(tag = "Ranklab")]
#[get("/coach/reviews?<params..>")]
pub async fn list(
  auth: Auth<Coach>,
  db_conn: DbConn,
  params: ListReviewsQuery,
) -> QueryResponse<Vec<ReviewView>> {
  let reviews: Vec<ReviewView> = db_conn
    .run(move |conn| {
      use crate::schema::reviews::dsl::*;
      use diesel::dsl::sql;
      use diesel::pg::Pg;
      use diesel::sql_types::Bool;

      let mut games_expression: Box<dyn BoxableExpression<reviews, Pg, SqlType = Bool>> =
        Box::new(sql::<Bool>("false"));

      for game in auth.0.games.into_iter() {
        games_expression = Box::new(
          games_expression.or(
            game_id
              .eq(game.game_id)
              .and(skill_level.lt(game.skill_level as i16)),
          ),
        );
      }

      let query = if params.pending.unwrap_or(false) {
        reviews
          .filter(state.eq(ReviewState::AwaitingReview).and(games_expression))
          .into_boxed()
      } else {
        reviews.filter(coach_id.eq(auth.0.id)).into_boxed()
      };

      query.load::<Review>(conn).unwrap()
    })
    .await
    .into_iter()
    .map(Into::into)
    .collect();

  Response::success(reviews)
}

#[derive(Deserialize, Validate, JsonSchema)]
pub struct UpdateReviewRequest {
  published: Option<bool>,
  taken: Option<bool>,
}

#[openapi(tag = "Ranklab")]
#[get("/coach/reviews/<id>")]
pub async fn get(id: Uuid, auth: Auth<Coach>, db_conn: DbConn) -> QueryResponse<ReviewView> {
  let review: ReviewView = db_conn
    .run(move |conn| {
      use crate::schema::reviews::dsl::{coach_id, id as review_id, reviews, state};
      reviews
        .filter(
          coach_id
            .eq(auth.0.id)
            .or(state.eq(ReviewState::AwaitingReview))
            .and(review_id.eq(id)),
        )
        .first::<Review>(conn)
    })
    .await?
    .into();

  Response::success(review)
}

#[openapi(tag = "Ranklab")]
#[put("/coach/reviews/<id>", data = "<review>")]
pub async fn update(
  id: Uuid,
  review: Json<UpdateReviewRequest>,
  auth: Auth<Coach>,
  db_conn: DbConn,
  stripe: Stripe,
) -> MutationResponse<ReviewView> {
  let auth_id = auth.0.id.clone();

  let existing_review = db_conn
    .run(move |conn| {
      use crate::schema::reviews::dsl::{coach_id, id as review_id, reviews, state};

      reviews
        .filter(
          review_id.eq(id).and(
            state
              .eq(ReviewState::AwaitingReview)
              .or(coach_id.eq(auth_id)),
          ),
        )
        .first::<Review>(conn)
    })
    .await?;

  let player_id = existing_review.player_id.clone();

  let player = db_conn
    .run(move |conn| {
      use crate::schema::players::dsl::*;

      players.filter(id.eq(player_id)).first::<Player>(conn)
    })
    .await?;

  if let Err(errors) = review.validate() {
    return Response::validation_error(errors);
  }

  if let Some(published) = review.published {
    if existing_review.state == ReviewState::Draft && published {
      let customer_id = player
        .stripe_customer_id
        .unwrap()
        .parse::<stripe::CustomerId>()
        .unwrap();

      stripe::PaymentIntent::create(
        &stripe.0 .0,
        stripe::CreatePaymentIntent {
          amount: 1000,
          currency: stripe::Currency::USD,
          description: Some("Review payment"),
          confirm: Some(true),
          customer: Some(customer_id),
          off_session: Some(stripe::PaymentIntentOffSession::Exists(true)),
          application_fee_amount: Some(123),
          transfer_data: Some(
            stripe::CreatePaymentIntentTransferData {
              destination: auth.0.stripe_account_id.unwrap(),
              amount: None,
            }
            .into(),
          ),
          error_on_requires_action: Some(true),
          automatic_payment_methods: None,
          capture_method: None,
          confirmation_method: None,
          expand: &[],
          mandate: None,
          mandate_data: None,
          metadata: None,
          on_behalf_of: None,
          payment_method: None,
          payment_method_data: None,
          payment_method_options: None,
          payment_method_types: None,
          receipt_email: None,
          return_url: None,
          setup_future_usage: None,
          shipping: None,
          statement_descriptor: None,
          statement_descriptor_suffix: None,
          transfer_group: None,
          use_stripe_sdk: None,
        },
      )
      .await
      .unwrap();

      let updated_review: ReviewView = db_conn
        .run(move |conn| {
          use crate::schema::reviews::dsl::state;

          diesel::update(&existing_review)
            .set(state.eq(ReviewState::Published))
            .get_result::<Review>(conn)
            .unwrap()
        })
        .await
        .into();

      return Response::success(updated_review);
    }
  }

  if let Some(true) = review.taken {
    if existing_review.state == ReviewState::AwaitingReview {
      let updated_review: ReviewView = db_conn
        .run(move |conn| {
          use crate::schema::reviews::dsl::*;

          diesel::update(&existing_review)
            .set((coach_id.eq(auth_id), state.eq(ReviewState::Draft)))
            .get_result::<Review>(conn)
            .unwrap()
        })
        .await
        .into();

      return Response::success(updated_review);
    }
  }

  Response::success(existing_review.into())
}
