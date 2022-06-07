use crate::data_types::ReviewState;
use crate::guards::{Auth, DbConn};
use crate::models::{Coach, Review, ReviewChangeset};
use crate::pagination::{Paginate, PaginatedResult};
use crate::response::{MutationResponse, QueryResponse, Response};
use crate::views::ReviewView;
use diesel::prelude::*;
use diesel::sql_types::Bool;
use rocket::serde::json::Json;
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

#[derive(FromForm, JsonSchema)]
pub struct ListReviewsQuery {
  pending: Option<bool>,
  page: Option<i64>,
}

#[openapi(tag = "Ranklab")]
#[get("/coach/reviews?<params..>")]
pub async fn list(
  auth: Auth<Coach>,
  db_conn: DbConn,
  params: ListReviewsQuery,
) -> QueryResponse<PaginatedResult<ReviewView>> {
  let (reviews, total_pages): (Vec<Review>, i64) = db_conn
    .run(move |conn| {
      Review::filter_for_coach(&auth.0, params.pending)
        .order(diesel::dsl::sql::<Bool>(
          "case \"state\"
            when 'awaiting_payment' then 1
            when 'awaiting_review' then 2
            when 'draft' then 3
            when 'published' then 4
            when 'accepted' then 5
            when 'refunded' then 5
          end,
          created_at desc",
        ))
        .paginate(params.page.unwrap_or(1))
        .load_and_count_pages::<Review>(conn)
        .unwrap()
    })
    .await;

  let review_views = reviews
    .into_iter()
    .map(|review| ReviewView::from(review, None))
    .collect();

  Response::success((review_views, total_pages).into())
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
    .run(move |conn| Review::find_for_coach(&id, &auth.0.id).first::<Review>(conn))
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
    .run(move |conn| Review::find_for_coach(&id, &auth.0.id).first::<Review>(conn))
    .await?;

  if let Err(errors) = review.validate() {
    return Response::validation_error(errors);
  }

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

      return Response::success(ReviewView::from(updated_review, None));
    }
  }

  if let Some(true) = review.taken {
    if existing_review.state == ReviewState::AwaitingReview {
      let updated_review = db_conn
        .run(move |conn| {
          diesel::update(&existing_review)
            .set(
              ReviewChangeset::default()
                .coach_id(Some(auth_id))
                .state(ReviewState::Draft),
            )
            .get_result::<Review>(conn)
            .unwrap()
        })
        .await;

      return Response::success(ReviewView::from(updated_review, None));
    }
  }

  Response::success(ReviewView::from(existing_review, None))
}
