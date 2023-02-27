use crate::guards::{Auth, DbConn, Jwt};
use crate::models::{Coach, Recording, Review};
use crate::response::{QueryResponse, Response};
use crate::schema::recordings;
use crate::views::RecordingView;
use diesel::prelude::*;
use rocket_okapi::openapi;
use uuid::Uuid;

#[openapi(tag = "Ranklab")]
#[get("/coach/recordings/<id>")]
pub async fn get(
  id: Uuid,
  auth: Auth<Jwt<Coach>>,
  db_conn: DbConn,
) -> QueryResponse<RecordingView> {
  let review: Review = db_conn
    .run(move |conn| {
      Review::find_by_recording_for_coach(&id, &auth.into_deep_inner().id).first(conn)
    })
    .await?;

  let recording_id = review.recording_id;

  let recording: Recording = db_conn
    .run(move |conn| {
      recordings::table
        .find(&recording_id)
        .get_result::<Recording>(conn)
    })
    .await?;

  Response::success(RecordingView::new(recording, Some(&review), None))
}
