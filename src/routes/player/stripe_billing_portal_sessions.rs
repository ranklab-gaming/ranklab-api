use crate::guards::Auth;
use crate::guards::Stripe;
use crate::models::Player;
use crate::response::{MutationResponse, Response};
use rocket::serde::json::Json;
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde;
use serde::{Deserialize, Serialize};

#[derive(Serialize, JsonSchema)]
pub struct BillingPortalLink {
  url: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct CreateBillingPortalSessionMutation {
  return_url: String,
}

#[derive(Serialize)]
struct BillingPortalSessionParams {
  return_url: String,
  customer: String,
}

#[derive(Deserialize)]
pub struct BillingPortalSession {
  pub url: String,
}

#[openapi(tag = "Ranklab")]
#[post("/player/stripe-billing-portal-sessions", data = "<body>")]
pub async fn create(
  auth: Auth<Player>,
  stripe: Stripe,
  body: Json<CreateBillingPortalSessionMutation>,
) -> MutationResponse<BillingPortalLink> {
  let billing_portal_session_params = BillingPortalSessionParams {
    return_url: body.return_url.clone(),
    customer: auth.0.stripe_customer_id.unwrap(),
  };

  let billing_portal_session: BillingPortalSession = stripe
    .0
     .0
    .post_form("/billing_portal/sessions", billing_portal_session_params)
    .await
    .unwrap();

  Response::success(BillingPortalLink {
    url: billing_portal_session.url,
  })
}
