use crate::db::DbConn;
use crate::guards::Auth;
use crate::models::{Recording, User};
use diesel::prelude::*;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket_okapi::openapi;
use uuid::Uuid;

#[openapi(tag = "Ranklab")]
#[get("/coach/recordings/<id>")]
pub async fn get(
  id: Uuid,
  _auth: Auth<User>,
  db_conn: DbConn,
) -> Result<Option<Json<Recording>>, Status> {
  let result = db_conn
    .run(move |conn| {
      use crate::schema::recordings;
      recordings::table.find(id).first::<Recording>(conn)
    })
    .await;

  match result {
    Ok(recording) => Ok(Some(Json(recording))),
    Err(diesel::result::Error::NotFound) => Ok(None),
    Err(error) => panic!("Error: {}", error),
  }
}
