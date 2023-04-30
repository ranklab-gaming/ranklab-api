use crate::intercom;
use crate::models::Avatar;
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
  pub slug: String,
  pub avatar_image_key: Option<String>,
}

impl From<Coach> for CoachView {
  fn from(coach: Coach) -> Self {
    CoachView::new(coach, None, None)
  }
}

impl CoachView {
  pub fn new(coach: Coach, config: Option<&Config>, avatar: Option<Avatar>) -> Self {
    let intercom_hash =
      config.and_then(|config| intercom::generate_user_hash(&coach.email, config));

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
      slug: coach.slug,
      intercom_hash,
      avatar_image_key: avatar.map(|avatar| avatar.processed_image_key).flatten(),
    }
  }
}
