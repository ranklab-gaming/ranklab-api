use crate::config::Config;
use crate::intercom::contacts::Contact;
use crate::intercom::RequestError;
use crate::response::{MutationError, MutationResponse, QueryResponse, Response, StatusResponse};
use crate::views::GameView;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde::Deserialize;
use std::collections::HashMap;
use validator::Validate;

#[derive(Deserialize, JsonSchema, Validate)]
pub struct CreateGameRequest {
  #[validate(email)]
  email: String,
  #[validate(length(min = 1))]
  name: String,
}

#[openapi(tag = "Ranklab")]
#[get("/games")]
pub async fn list() -> QueryResponse<Vec<GameView>> {
  Response::success(crate::games::all().iter().map(|g| g.into()).collect())
}

#[openapi(tag = "Ranklab")]
#[post("/games", data = "<game>")]
pub async fn create(
  game: Json<CreateGameRequest>,
  config: &State<Config>,
) -> MutationResponse<StatusResponse> {
  if let Err(errors) = game.validate() {
    return Response::validation_error(errors);
  }

  let mut custom_attributes = HashMap::new();

  custom_attributes.insert("Requested Game".to_string(), game.name.clone());

  Contact::new(game.email.clone(), custom_attributes)
    .create(config)
    .await
    .map_err(|err| match err {
      RequestError::Conflict(_) => MutationError::Status(Status::UnprocessableEntity),
      RequestError::ServerError(err) => MutationError::InternalServerError(err.into()),
    })?;

  Response::status(Status::Ok)
}
