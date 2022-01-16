use crate::models::Game;
use crate::response::{QueryResponse, Response};
use rocket_okapi::openapi;

#[openapi(tag = "Ranklab")]
#[get("/user/games")]
pub async fn list() -> QueryResponse<&'static Vec<Box<dyn Game>>> {
  Response::success(crate::games::all())
}
