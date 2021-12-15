use crate::games::Game;
use crate::guards::Auth;
use crate::models::User;
use rocket::serde::json::Json;
use rocket_okapi::openapi;

#[openapi(tag = "Ranklab")]
#[get("/games")]
pub async fn list(_auth: Auth<User>) -> Json<Vec<Box<dyn Game>>> {
  Json(crate::games::all())
}
