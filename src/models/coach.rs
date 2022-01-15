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
  pub auth0_id: String,
}
