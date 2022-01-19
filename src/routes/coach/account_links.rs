use crate::guards::Auth;
use crate::guards::Stripe;
use crate::models::Coach;
use crate::response::{MutationResponse, Response};
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde;
use serde::{Deserialize, Serialize};

#[derive(Serialize, JsonSchema)]
pub struct AccountLink {
  url: String,
}

#[derive(Deserialize)]
struct StripeAccountLink {
  url: String,
}

#[derive(Serialize)]
struct CreateAccountLink {
  account: String,
  refresh_url: String,
  return_url: String,
  #[serde(rename = "type")]
  type_: String,
}

#[openapi(tag = "Ranklab")]
#[post("/coach/account-links")]
pub async fn create(auth: Auth<Coach>, stripe: Stripe) -> MutationResponse<AccountLink> {
  let account_link = stripe
    .0
    .post_form::<StripeAccountLink, CreateAccountLink>(
      "/account_links",
      CreateAccountLink {
        account: auth.0.stripe_account_id.unwrap(),
        refresh_url: "http://example.com".to_owned(),
        return_url: "http://example.com".to_owned(),
        type_: "account_onboarding".to_owned(),
      },
    )
    .await
    .unwrap();

  Response::success(AccountLink {
    url: account_link.url,
  })
}
