use crate::guards::{Auth, Jwt, Stripe};
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
pub struct CreateBillingPortalSession {
  return_url: String,
}

#[derive(Serialize)]
struct BillingPortalSessionParams {
  return_url: String,
  customer: String,
}

#[derive(Deserialize)]
struct BillingPortalSession {
  url: String,
}

#[openapi(tag = "Ranklab")]
#[post("/player/stripe-billing-portal-sessions", data = "<body>")]
pub async fn create(
  auth: Auth<Jwt<Player>>,
  stripe: Stripe,
  body: Json<CreateBillingPortalSession>,
) -> MutationResponse<BillingPortalLink> {
  let billing_portal_session_params = BillingPortalSessionParams {
    return_url: body.return_url.clone(),
    customer: auth.into_deep_inner().stripe_customer_id,
  };

  let billing_portal_session: BillingPortalSession = stripe
    .into_inner()
    .post_form("/billing_portal/sessions", billing_portal_session_params)
    .await
    .unwrap();

  Response::success(BillingPortalLink {
    url: billing_portal_session.url,
  })
}
