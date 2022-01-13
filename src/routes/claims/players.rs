use crate::db::DbConn;
use crate::guards::auth::Claims;
use crate::guards::Auth;
use crate::models::Player;
use crate::response::{MutationResponse, Response};
use diesel::prelude::*;
use rocket::serde::json::Json;
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde::Deserialize;
use validator::Validate;

#[derive(Deserialize, Validate, JsonSchema)]
pub struct CreatePlayerRequest {
  #[validate(length(min = 1))]
  name: String,
}

#[openapi(tag = "Ranklab")]
#[post("/claims/players", data = "<player>")]
pub async fn create(
  player: Json<CreatePlayerRequest>,
  auth: Auth<Claims>,
  db_conn: DbConn,
) -> MutationResponse<Player> {
  if let Err(errors) = player.validate() {
    return Response::validation_error(errors);
  }

  let player = db_conn
    .run(move |conn| {
      use crate::schema::players::dsl::*;

      diesel::insert_into(players)
        .values((
          email.eq(auth.0.email.clone()),
          name.eq(player.name.clone()),
          auth0_id.eq(auth.0.sub.clone()),
        ))
        .get_result(conn)
        .unwrap()
    })
    .await;

  Response::success(player)
}
