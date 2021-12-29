use crate::games::Game;
use crate::guards::Auth;
use crate::models::Player;
use rocket::serde::json::Json;
use rocket_okapi::openapi;

#[openapi(tag = "Ranklab")]
#[get("/games")]
pub async fn list(_auth: Auth<Player>) -> Json<Vec<Box<dyn Game>>> {
  Json(crate::games::all())
}
