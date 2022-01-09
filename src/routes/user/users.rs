use crate::guards::Auth;
use crate::models::User;
use rocket::serde::json::Json;
use rocket_okapi::openapi;

#[openapi(tag = "Ranklab")]
#[get("/user/users/me")]
pub async fn get_me(auth: Auth<User>) -> Json<User> {
  Json(auth.0)
}
