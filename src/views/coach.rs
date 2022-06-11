use crate::data_types::UserGame;
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
  pub games: Vec<UserGame>,
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
      games: coach.games.into_iter().map(|game| game.unwrap()).collect(),
      country: coach.country,
      can_review: coach.stripe_payouts_enabled,
      stripe_details_submitted: coach.stripe_details_submitted,
    }
  }
}
