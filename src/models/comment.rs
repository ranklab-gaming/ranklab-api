use schemars::JsonSchema;
use serde::Serialize;
use uuid::Uuid;

#[derive(Queryable, Serialize, JsonSchema)]
pub struct Comment {
  pub id: Uuid,
  pub review_id: Uuid,
  pub user_id: Uuid,
  pub body: String,
  pub video_timestamp: i32,
  pub drawing: String,
}
