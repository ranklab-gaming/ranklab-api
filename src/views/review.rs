use crate::data_types::ReviewState;
use crate::models::{Coach, Review};
use schemars::JsonSchema;
use serde::Serialize;
use uuid::Uuid;

use super::CoachView;

#[derive(Serialize, JsonSchema)]
#[serde(rename = "Review")]
pub struct ReviewView {
  pub id: Uuid,
  pub player_id: Uuid,
  pub coach_id: Uuid,
  pub title: String,
  pub recording_id: Uuid,
  pub game_id: String,
  pub skill_level: i16,
  pub notes: String,
  pub state: ReviewState,
  pub created_at: chrono::NaiveDateTime,
  pub stripe_client_secret: Option<String>,
  pub coach: Option<CoachView>,
}

impl ReviewView {
  pub fn new(
    review: Review,
    payment_intent: Option<stripe::PaymentIntent>,
    coach: Option<Coach>,
  ) -> Self {
    ReviewView {
      id: review.id,
      player_id: review.player_id,
      coach_id: review.coach_id,
      title: review.title,
      recording_id: review.recording_id,
      game_id: review.game_id,
      skill_level: review.skill_level,
      notes: review.notes,
      state: review.state,
      created_at: review.created_at,
      stripe_client_secret: match payment_intent {
        Some(payment_intent) => Some(payment_intent.client_secret.unwrap()),
        None => None,
      },
      coach: match coach {
        Some(coach) => Some(coach.into()),
        None => None,
      },
    }
  }
}
