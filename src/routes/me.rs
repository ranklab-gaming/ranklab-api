use crate::guards::Auth;
use crate::models::Player;
use rocket::serde::json::Json;
use rocket_okapi::openapi;

#[openapi(tag = "Ranklab")]
#[get("/me")]
pub async fn get_me(auth: Auth<Player>) -> Json<Player> {
  Json(auth.0)
}
