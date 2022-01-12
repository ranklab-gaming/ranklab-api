use crate::games::Game;
use rocket::serde::json::Json;
use rocket_okapi::openapi;

#[openapi(tag = "Ranklab")]
#[get("/user/games")]
pub async fn list() -> Json<Vec<Box<dyn Game>>> {
  Json(crate::games::all())
}
