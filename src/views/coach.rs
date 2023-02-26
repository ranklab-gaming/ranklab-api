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
  pub reviews_enabled: bool,
  pub payouts_enabled: bool,
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
      payouts_enabled: coach.stripe_payouts_enabled,
      reviews_enabled: coach.stripe_details_submitted,
    }
  }
}
