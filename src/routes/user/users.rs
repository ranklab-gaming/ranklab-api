use crate::guards::Auth;
use crate::models::User;
use crate::response::{QueryResponse, Response};
use crate::views::UserView;
use rocket_okapi::openapi;

#[openapi(tag = "Ranklab")]
#[get("/user/users/me")]
pub async fn get_me(auth: Auth<User>) -> QueryResponse<UserView> {
  Response::success(auth.0.into())
}
