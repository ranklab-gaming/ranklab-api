use crate::guards::{Auth, DbConn};
use crate::models::{Coach, Recording, Review};
use crate::response::{QueryResponse, Response};
use crate::views::RecordingView;
use diesel::prelude::*;
use rocket_okapi::openapi;
use uuid::Uuid;

#[openapi(tag = "Ranklab")]
#[get("/coach/recordings/<id>")]
pub async fn get(id: Uuid, auth: Auth<Coach>, db_conn: DbConn) -> QueryResponse<RecordingView> {
  let review: Review = db_conn
    .run(move |conn| Review::find_by_recording_for_coach(&id, &auth.0.id).first(conn))
    .await?;

  let recording: RecordingView = db_conn
    .run(move |conn| Recording::find(&review.recording_id).get_result::<Recording>(conn))
    .await?
    .into();

  Response::success(recording)
}
