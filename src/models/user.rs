use schemars::JsonSchema;
use serde::Serialize;
use uuid::Uuid;

#[derive(Queryable, Serialize, JsonSchema)]
pub struct User {
  pub id: Uuid,
  pub auth0_id: String,
}
