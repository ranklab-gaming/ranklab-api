use crate::db::DbConn;
use crate::guards::Auth;
use crate::models::{Coach, User};
use crate::response::Response;
use diesel::prelude::*;
use rocket::serde::json::Json;
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

#[derive(Deserialize, Validate, JsonSchema)]
pub struct CreateCoachRequest {
  #[validate(email)]
  email: String,
  #[validate(length(min = 1))]
  name: String,
  #[validate(length(min = 1))]
  bio: String,
  game_id: Uuid,
}

#[openapi(tag = "Ranklab")]
#[post("/coaches", data = "<coach>")]
pub async fn create(
  coach: Json<CreateCoachRequest>,
  auth: Auth<User>,
  db_conn: DbConn,
) -> Response<Coach> {
  if let Err(errors) = coach.validate() {
    return Response::ValidationErrors(errors);
  }

  let coach = db_conn
    .run(move |conn| {
      use crate::schema::coaches::dsl::*;

      diesel::insert_into(coaches)
        .values((
          email.eq(coach.email.clone()),
          name.eq(coach.name.clone()),
          bio.eq(coach.bio.clone()),
          game_id.eq(coach.game_id.clone()),
          user_id.eq(auth.0.id.clone()),
        ))
        .get_result(conn)
        .unwrap()
    })
    .await;

  Response::Success(coach)
}
