use crate::response::{QueryResponse, Response};
use crate::views::GameView;
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde::Deserialize;
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
