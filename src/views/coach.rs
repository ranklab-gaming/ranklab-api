use crate::intercom;
use crate::{config::Config, models::Coach};
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
  pub game_id: String,
  pub price: i32,
  pub country: String,
  pub reviews_enabled: bool,
  pub payouts_enabled: bool,
  pub emails_enabled: bool,
  pub intercom_hash: Option<String>,
}

impl From<Coach> for CoachView {
  fn from(coach: Coach) -> Self {
    CoachView::new(coach, None)
  }
}

impl CoachView {
  pub fn new(coach: Coach, config: Option<&Config>) -> Self {
    let intercom_hash = config.and_then(|config| {
      config
        .intercom_verification_secret
        .as_ref()
        .map(|secret| intercom::generate_user_hash(coach.id, secret))
    });

    CoachView {
      id: coach.id,
      name: coach.name,
      email: coach.email,
      bio: coach.bio,
      game_id: coach.game_id,
      price: coach.price,
      country: coach.country,
      payouts_enabled: coach.stripe_payouts_enabled,
      reviews_enabled: coach.stripe_details_submitted,
      emails_enabled: coach.emails_enabled,
      intercom_hash,
    }
  }
}
