use schemars::JsonSchema;
use serde::Serialize;
use uuid::Uuid;

#[derive(Queryable, Serialize, JsonSchema)]
pub struct Coach {
  pub id: Uuid,
  pub user_id: Uuid,
  pub name: String,
  pub email: String,
  pub bio: String,
  pub game: String,
}
