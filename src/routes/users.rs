use crate::models::User;
use rocket::serde::json::Json;
use rocket::serde::Deserialize;
use rocket::Route;

#[derive(Deserialize)]
struct UserRequest {
  auth0_id: String,
}

#[post("/", data = "<user>")]
fn create_user(user: Json<UserRequest>) -> Json<User> {
  Json(User {
    id: user.auth0_id.clone(),
    auth0_id: user.auth0_id.clone(),
  })
}

pub fn build() -> Vec<Route> {
  routes![create_user]
}
