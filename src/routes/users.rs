use crate::guards::Auth;
use crate::models::User;
use rocket::serde::json::Json;
use rocket_okapi::openapi;

#[openapi(tag = "Ranklab")]
#[get("/users/current")]
pub async fn get_current(auth: Auth<User>) -> Json<User> {
    Json(auth.0)
}
