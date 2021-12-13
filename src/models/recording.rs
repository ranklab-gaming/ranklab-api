use schemars::JsonSchema;
use serde::Serialize;
use uuid::Uuid;

#[derive(Queryable, Serialize, JsonSchema)]
pub struct Recording {
  pub id: Uuid,
  pub user_id: Uuid,
  pub video_key: String,
  pub upload_url: String,
  pub uploaded: bool,
}
