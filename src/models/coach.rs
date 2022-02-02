use crate::models::UserGame;
use schemars::JsonSchema;
use serde::Serialize;
use uuid::Uuid;

#[derive(Queryable, Serialize, JsonSchema)]
pub struct Coach {
  pub id: Uuid,
  pub name: String,
  pub email: String,
  pub bio: String,
  pub games: Vec<UserGame>,
  #[schemars(skip)]
  #[serde(skip_serializing)]
  pub auth0_id: String,
  #[schemars(skip)]
  #[serde(skip_serializing)]
  pub stripe_account_id: Option<String>,
  pub can_review: bool,
  pub submitted_stripe_details: bool,
  pub country: String,
}
