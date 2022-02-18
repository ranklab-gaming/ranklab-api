use crate::data_types::ReviewState;
use crate::guards::Auth;
use crate::guards::DbConn;
use crate::models::{Coach, Review};
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
    .map(|review| ReviewView::from(review, None))
    .collect();

  Response::success(reviews)
}

#[derive(Deserialize, Validate, JsonSchema)]
#[schemars(rename = "CoachUpdateReviewRequest")]
pub struct UpdateReviewRequest {
  published: Option<bool>,
  taken: Option<bool>,
}

#[openapi(tag = "Ranklab")]
#[get("/coach/reviews/<id>")]
pub async fn get(id: Uuid, auth: Auth<Coach>, db_conn: DbConn) -> QueryResponse<ReviewView> {
  let review = db_conn
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
    .await?;

  Response::success(ReviewView::from(review, None))
}

#[openapi(tag = "Ranklab")]
#[put("/coach/reviews/<id>", data = "<review>")]
pub async fn update(
  id: Uuid,
  review: Json<UpdateReviewRequest>,
  auth: Auth<Coach>,
  db_conn: DbConn,
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

  if let Err(errors) = review.validate() {
    return Response::validation_error(errors);
  }

  if let Some(true) = review.published {
    if existing_review.state == ReviewState::Draft {
      let updated_review = db_conn
        .run(move |conn| {
          use crate::schema::reviews::dsl::state;

          diesel::update(&existing_review)
            .set(state.eq(ReviewState::Published))
            .get_result::<Review>(conn)
            .unwrap()
        })
        .await;

      return Response::success(ReviewView::from(updated_review, None));
    }
  }

  if let Some(true) = review.taken {
    if existing_review.state == ReviewState::AwaitingReview {
      let updated_review = db_conn
        .run(move |conn| {
          use crate::schema::reviews::dsl::*;

          diesel::update(&existing_review)
            .set((coach_id.eq(auth_id), state.eq(ReviewState::Draft)))
            .get_result::<Review>(conn)
            .unwrap()
        })
        .await;

      return Response::success(ReviewView::from(updated_review, None));
    }
  }

  Response::success(ReviewView::from(existing_review, None))
}
