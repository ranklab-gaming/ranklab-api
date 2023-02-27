use crate::guards::{Auth, Jwt, Stripe};
use crate::models::Coach;
use crate::response::{MutationResponse, Response};
use rocket::serde::json::Json;
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde;
use serde::{Deserialize, Serialize};

#[derive(Serialize, JsonSchema)]
pub struct AccountLink {
  url: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct CreateAccountLinkMutation {
  refresh_url: String,
  return_url: String,
}

#[openapi(tag = "Ranklab")]
#[post("/coach/stripe-account-links", data = "<body>")]
pub async fn create(
  auth: Auth<Jwt<Coach>>,
  stripe: Stripe,
  body: Json<CreateAccountLinkMutation>,
) -> MutationResponse<AccountLink> {
  let mut account_link_params = stripe::CreateAccountLink::new(
    auth
      .into_deep_inner()
      .stripe_account_id
      .parse::<stripe::AccountId>()
      .unwrap(),
    stripe::AccountLinkType::AccountOnboarding,
  );

  account_link_params.refresh_url = Some(body.refresh_url.as_str());
  account_link_params.return_url = Some(body.return_url.as_str());

  let account_link = stripe::AccountLink::create(&stripe.into_inner(), account_link_params)
    .await
    .unwrap();

  Response::success(AccountLink {
    url: account_link.url,
  })
}
