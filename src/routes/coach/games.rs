use crate::response::{QueryResponse, Response};
use crate::views::GameView;
use rocket_okapi::openapi;

#[openapi(tag = "Ranklab")]
#[get("/coach/games")]
pub async fn list() -> QueryResponse<Vec<GameView>> {
  Response::success(crate::games::all().iter().map(Into::into).collect())
}
