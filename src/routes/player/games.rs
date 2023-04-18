use crate::config::Config;
use crate::guards::DbConn;
use crate::intercom::contacts::Contact;
use crate::intercom::RequestError;
use crate::response::{MutationError, MutationResponse, QueryResponse, Response, StatusResponse};
use crate::schema::coaches::game_id;
use crate::views::GameView;
use diesel::prelude::*;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde::Deserialize;
use std::collections::HashMap;
use validator::Validate;

#[openapi(tag = "Ranklab")]
#[get("/player/games")]
pub async fn list(db_conn: DbConn) -> QueryResponse<Vec<GameView>> {
  let game_ids = db_conn
    .run(move |conn| {
      crate::schema::coaches::table
        .select(game_id)
        .distinct()
        .load::<String>(conn)
    })
    .await?;

  let game_id_strs: Vec<&str> = game_ids.iter().map(|s| s.as_str()).collect();

  Response::success(
    crate::games::filter(game_id_strs)
      .iter()
      .map(|g| Into::into(*g))
      .collect(),
  )
}

#[derive(Deserialize, JsonSchema, Validate)]
pub struct CreateGameRequest {
  #[validate(email)]
  email: String,
  #[validate(length(min = 1))]
  name: String,
}

#[openapi(tag = "Ranklab")]
#[post("/player/games", data = "<game_request>")]
pub async fn create(
  game_request: Json<CreateGameRequest>,
  config: &State<Config>,
) -> MutationResponse<StatusResponse> {
  if let Err(errors) = game_request.validate() {
    return Response::validation_error(errors);
  }

  let mut custom_attributes = HashMap::new();

  custom_attributes.insert("Requested Game".to_string(), game_request.name.clone());

  Contact::new(game_request.email.clone(), custom_attributes)
    .create(config)
    .await
    .map_err(|err| match err {
      RequestError::Conflict(_) => MutationError::Status(Status::UnprocessableEntity),
      RequestError::ServerError(err) => MutationError::InternalServerError(err.into()),
    })?;

  Response::status(Status::Ok)
}
