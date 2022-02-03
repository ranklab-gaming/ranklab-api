use crate::response::{QueryResponse, Response};
use crate::views::GameView;
use rocket_okapi::openapi;

#[openapi(tag = "Ranklab")]
#[get("/user/games")]
pub async fn list() -> QueryResponse<Vec<GameView>> {
  Response::success(crate::games::all().into_iter().map(Into::into).collect())
}
