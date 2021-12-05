use schemars::JsonSchema;
use serde::Serialize;
use uuid::Uuid;

#[derive(Queryable, Serialize, JsonSchema)]
pub struct Recording {
  pub id: Uuid,
  pub user_id: Uuid,
  pub extension: String,
  pub uploaded: bool,
}
