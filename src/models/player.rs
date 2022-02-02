use crate::models::UserGame;
use schemars::JsonSchema;
use serde::Serialize;
use uuid::Uuid;

#[derive(Queryable, Serialize, JsonSchema)]
pub struct Player {
  pub id: Uuid,
  pub auth0_id: String,
  pub name: String,
  pub email: String,
  pub games: Vec<UserGame>,
  #[schemars(skip)]
  #[serde(skip_serializing)]
  pub stripe_customer_id: Option<String>,
}
