use crate::db::DbConn;
use crate::guards::Auth;
use crate::models::{Coach, Recording, Review};
use crate::response::Response;
use diesel::prelude::*;
use rocket_okapi::openapi;
use uuid::Uuid;

#[openapi(tag = "Ranklab")]
#[get("/coach/recordings/<recording_id_param>")]
pub async fn get(
  recording_id_param: Uuid,
  auth: Auth<Coach>,
  db_conn: DbConn,
) -> Response<Recording> {
  let review = db_conn
    .run(move |conn| {
      use crate::schema::reviews::dsl::*;
      reviews
        .filter(
          coach_id
            .eq(auth.0.id)
            .and(recording_id.eq(recording_id_param)),
        )
        .first::<Review>(conn)
    })
    .await?;

  let recording = db_conn
    .run(move |conn| {
      use crate::schema::recordings::dsl::*;
      recordings.filter(id.eq(review.recording_id)).first(conn)
    })
    .await?;

  Response::Success(recording)
}
