use schemars::JsonSchema;
use serde::Serialize;
use uuid::Uuid;

#[derive(Queryable, Serialize, JsonSchema)]
pub struct Review {
  pub id: Uuid,
  pub player_id: Uuid,
  pub coach_id: Option<Uuid>,
  pub title: String,
  pub recording_id: Uuid,
  pub game_id: String,
  pub skill_level: i16,
  pub notes: String,
  pub published: bool,
}
