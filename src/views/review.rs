use crate::models::Review;
use schemars::JsonSchema;
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize, JsonSchema)]
#[serde(rename = "Review")]
pub struct ReviewView {
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

impl From<Review> for ReviewView {
  fn from(review: Review) -> Self {
    ReviewView {
      id: review.id,
      player_id: review.player_id,
      coach_id: review.coach_id,
      title: review.title,
      recording_id: review.recording_id,
      game_id: review.game_id,
      skill_level: review.skill_level,
      notes: review.notes,
      published: review.published,
    }
  }
}
