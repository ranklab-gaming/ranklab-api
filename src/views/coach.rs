use crate::models::Coach;
use schemars::JsonSchema;
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize, JsonSchema)]
#[serde(rename = "Coach")]
pub struct CoachView {
  pub id: Uuid,
  pub name: String,
  pub email: String,
  pub bio: String,
  pub game_ids: Vec<String>,
  pub country: String,
  pub can_review: bool,
  pub stripe_details_submitted: bool,
}

impl From<Coach> for CoachView {
  fn from(coach: Coach) -> Self {
    CoachView {
      id: coach.id,
      name: coach.name,
      email: coach.email,
      bio: coach.bio,
      game_ids: coach.game_ids.into_iter().map(|id| id.unwrap()).collect(),
      country: coach.country,
      can_review: coach.stripe_payouts_enabled,
      stripe_details_submitted: coach.stripe_details_submitted,
    }
  }
}
