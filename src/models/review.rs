use schemars::JsonSchema;
use serde::Serialize;
use uuid::Uuid;

#[derive(Queryable, Serialize, JsonSchema)]
pub struct Review {
  pub id: Uuid,
  pub user_id: Uuid,
  pub coach_id: Option<Uuid>,
  pub title: String,
  pub video_url: String,
  pub game: String,
}
