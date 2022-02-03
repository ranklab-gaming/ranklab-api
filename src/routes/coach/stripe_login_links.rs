use crate::guards::Auth;
use crate::guards::Stripe;
use crate::models::Coach;
use crate::response::{MutationResponse, Response};
use rocket::serde::json::Json;
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde;
use serde::{Deserialize, Serialize};
use stripe::CreateLoginLinkExt;

#[derive(Serialize, JsonSchema)]
pub struct LoginLink {
  url: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct CreateLoginLinkMutation {
  return_url: String,
}

#[openapi(tag = "Ranklab")]
#[post("/coach/stripe-login-links", data = "<body>")]
pub async fn create(
  auth: Auth<Coach>,
  stripe: Stripe,
  body: Json<CreateLoginLinkMutation>,
) -> MutationResponse<LoginLink> {
  let account_id = auth
    .0
    .stripe_account_id
    .unwrap()
    .parse::<stripe::AccountId>()
    .unwrap();

  let login_link = stripe::LoginLink::create(&stripe.0 .0, &account_id, body.return_url.as_str())
    .await
    .unwrap();

  Response::success(LoginLink {
    url: login_link.url,
  })
}
