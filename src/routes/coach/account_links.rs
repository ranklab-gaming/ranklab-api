use crate::guards::Auth;
use crate::models::Coach;
use crate::response::{MutationResponse, Response};
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde::Serialize;
use crate::config::Config;
use rocket::State;

#[derive(Serialize, JsonSchema)]
pub struct AccountLink {
  url: String,
}

#[openapi(tag = "Ranklab")]
#[post("/coach/account-links")]
pub async fn create(_auth: Auth<Coach>, config: &State<Config>) -> MutationResponse<AccountLink> {
  let client = stripe::Client::new(config.stripe_secret.clone());

  Response::success(AccountLink {
    url: "".to_string(),
  })
}
