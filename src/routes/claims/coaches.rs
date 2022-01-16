use crate::db::DbConn;
use crate::guards::auth::Claims;
use crate::guards::Auth;
use crate::models::Coach;
use crate::models::UserGame;
use crate::response::{MutationResponse, Response};
use diesel::prelude::*;
use rocket::serde::json::Json;
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde::Deserialize;
use validator::Validate;

#[derive(Deserialize, Validate, JsonSchema)]
pub struct CreateCoachRequest {
  #[validate(length(min = 1))]
  name: String,
  #[validate(length(min = 1))]
  bio: String,
  games: Vec<UserGame>,
}

#[openapi(tag = "Ranklab")]
#[post("/claims/coaches", data = "<coach>")]
pub async fn create(
  coach: Json<CreateCoachRequest>,
  auth: Auth<Claims>,
  db_conn: DbConn,
) -> MutationResponse<Coach> {
  if let Err(errors) = coach.validate() {
    return Response::validation_error(errors);
  }

  let coach = db_conn
    .run(move |conn| {
      use crate::schema::coaches::dsl::*;

      diesel::insert_into(coaches)
        .values((
          email.eq(auth.0.email.clone()),
          name.eq(coach.name.clone()),
          bio.eq(coach.bio.clone()),
          games.eq(coach.games.clone()),
          auth0_id.eq(auth.0.sub.clone()),
        ))
        .get_result(conn)
        .unwrap()
    })
    .await;

  Response::success(coach)
}
