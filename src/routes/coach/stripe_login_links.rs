use crate::guards::{Auth, Jwt, Stripe};
use crate::models::Coach;
use crate::response::{MutationResponse, Response};
use rocket::serde::json::Json;
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde;
use serde::{Deserialize, Serialize};

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
  auth: Auth<Jwt<Coach>>,
  stripe: Stripe,
  body: Json<CreateLoginLinkMutation>,
) -> MutationResponse<LoginLink> {
  let account_id = auth
    .into_deep_inner()
    .stripe_account_id
    .parse::<stripe::AccountId>()
    .unwrap();

  let login_link =
    stripe::LoginLink::create(&stripe.into_inner(), &account_id, body.return_url.as_str())
      .await
      .unwrap();

  Response::success(LoginLink {
    url: login_link.url,
  })
}
