use crate::guards::Auth;
use crate::models::User;
use crate::response;
use crate::response::QueryResponse;
use rocket_okapi::openapi;

#[openapi(tag = "Ranklab")]
#[get("/user/users/me")]
pub async fn get_me(auth: Auth<User>) -> QueryResponse<User> {
  response::success(auth.0)
}
