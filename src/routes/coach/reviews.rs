use crate::data_types::ReviewState;
use crate::guards::{Auth, DbConn, Jwt};
use crate::models::{Coach, Recording, Review, ReviewChangeset};
use crate::pagination::{Paginate, PaginatedResult};
use crate::response::{MutationResponse, QueryResponse, Response};
use crate::views::{ReviewView, ReviewViewOptions};
use diesel::prelude::*;
use rocket::serde::json::Json;
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

#[derive(FromForm, JsonSchema)]
pub struct ListReviewsQuery {
  page: Option<i64>,
  archived: Option<bool>,
}

#[openapi(tag = "Ranklab")]
#[get("/coach/reviews?<params..>")]
pub async fn list(
  auth: Auth<Jwt<Coach>>,
  db_conn: DbConn,
  params: ListReviewsQuery,
) -> QueryResponse<PaginatedResult<ReviewView>> {
  let coach = auth.into_deep_inner();
  let cloned_coach = coach.clone();

  let paginated_reviews: PaginatedResult<Review> = db_conn
    .run(move |conn| {
      Review::filter_for_coach(&coach, params.archived.unwrap_or(false))
        .paginate(params.page.unwrap_or(1))
        .load_and_count_pages::<Review>(conn)
        .unwrap()
    })
    .await;

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

  let review_views: Vec<ReviewView> = paginated_reviews
    .records
    .clone()
    .into_iter()
    .map(|review| {
      let recording_id = review.recording_id;

      ReviewView::new(
        review,
        ReviewViewOptions {
          payment_intent: None,
          coach: Some(cloned_coach.clone()),
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

#[derive(Deserialize, Validate, JsonSchema)]
#[schemars(rename = "CoachUpdateReviewRequest")]
pub struct UpdateReviewRequest {
  published: Option<bool>,
  taken: Option<bool>,
}

#[openapi(tag = "Ranklab")]
#[get("/coach/reviews/<id>")]
pub async fn get(id: Uuid, auth: Auth<Jwt<Coach>>, db_conn: DbConn) -> QueryResponse<ReviewView> {
  let coach = auth.into_deep_inner();
  let coach_id = coach.id;

  let review = db_conn
    .run(move |conn| Review::find_for_coach(&id, &coach_id).first::<Review>(conn))
    .await?;

  let recording_id = review.recording_id;

  let recording = db_conn
    .run(move |conn| Recording::find_by_id(&recording_id).first::<Recording>(conn))
    .await?;

  Response::success(ReviewView::new(
    review,
    ReviewViewOptions {
      payment_intent: None,
      coach: Some(coach),
      recording: Some(recording),
    },
  ))
}

#[openapi(tag = "Ranklab")]
#[put("/coach/reviews/<id>", data = "<review>")]
pub async fn update(
  id: Uuid,
  review: Json<UpdateReviewRequest>,
  auth: Auth<Jwt<Coach>>,
  db_conn: DbConn,
) -> MutationResponse<ReviewView> {
  if let Err(errors) = review.validate() {
    return Response::validation_error(errors);
  }

  let existing_review = db_conn
    .run(move |conn| Review::find_for_coach(&id, &auth.into_deep_inner().id).first::<Review>(conn))
    .await?;

  if let Some(true) = review.published {
    if existing_review.state == ReviewState::Draft {
      let updated_review = db_conn
        .run(move |conn| {
          diesel::update(&existing_review)
            .set(ReviewChangeset::default().state(ReviewState::Published))
            .get_result::<Review>(conn)
            .unwrap()
        })
        .await;

      return Response::success(updated_review.into());
    }
  }

  if let Some(true) = review.taken {
    if existing_review.state == ReviewState::AwaitingReview {
      let updated_review = db_conn
        .run(move |conn| {
          diesel::update(&existing_review)
            .set(ReviewChangeset::default().state(ReviewState::Draft))
            .get_result::<Review>(conn)
            .unwrap()
        })
        .await;

      return Response::success(updated_review.into());
    }
  }

  Response::success(existing_review.into())
}
