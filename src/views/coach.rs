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
  pub details_submitted: bool,
  pub payouts_enabled: bool,
  pub emails_enabled: bool,
  pub intercom_hash: Option<String>,
  pub slug: String,
  pub avatar_image_key: Option<String>,
  pub approved: bool,
  pub bio_text: String,
  pub reviews_count: i32,
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

    let bio = coach.bio.clone();

    CoachView {
      id: coach.id,
      name: coach.name,
      email: coach.email,
      bio: coach.bio,
      game_id: coach.game_id,
      price: coach.price,
      country: coach.country,
      payouts_enabled: coach.stripe_payouts_enabled,
      details_submitted: coach.stripe_details_submitted,
      approved: coach.approved,
      emails_enabled: coach.emails_enabled,
      slug: coach.slug,
      intercom_hash,
      avatar_image_key: avatar.and_then(|avatar| avatar.processed_image_key),
      bio_text: html2text::from_read(bio.as_bytes(), 100),
      reviews_count: coach.reviews_count,
    }
  }
}
