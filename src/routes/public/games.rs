use crate::games::Game;
use crate::response;
use crate::response::QueryResponse;
use rocket_okapi::openapi;

#[openapi(tag = "Ranklab")]
#[get("/user/games")]
pub async fn list() -> QueryResponse<Vec<Box<dyn Game>>> {
  response::success(crate::games::all())
}
