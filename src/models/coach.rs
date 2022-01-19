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
  #[serde(skip_serializing)]
  pub auth0_id: String,
  #[serde(skip_serializing)]
  pub stripe_account_id: Option<String>,
}
