use crate::data_types::ReviewState;
use crate::guards::Auth;
use crate::guards::DbConn;
use crate::models::{Coach, Recording, Review};
use crate::response::{QueryResponse, Response};
use crate::views::RecordingView;
use diesel::prelude::*;
use rocket_okapi::openapi;
use uuid::Uuid;

#[openapi(tag = "Ranklab")]
#[get("/coach/recordings/<id>")]
pub async fn get(id: Uuid, auth: Auth<Coach>, db_conn: DbConn) -> QueryResponse<RecordingView> {
  let review = db_conn
    .run(move |conn| {
      use crate::schema::reviews::dsl::{coach_id, recording_id, reviews, state};

      reviews
        .filter(
          coach_id
            .eq(auth.0.id)
            .or(state.eq(ReviewState::AwaitingReview))
            .and(recording_id.eq(id)),
        )
        .first::<Review>(conn)
    })
    .await?;

  let recording: RecordingView = db_conn
    .run(move |conn| {
      use crate::schema::recordings::dsl::*;
      recordings
        .filter(id.eq(review.recording_id))
        .first::<Recording>(conn)
    })
    .await?
    .into();

  Response::success(recording)
}
