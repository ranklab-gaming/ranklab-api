use crate::models::ReviewIntent;
use schemars::JsonSchema;
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize, JsonSchema)]
#[serde(rename = "ReviewIntent")]
pub struct ReviewIntentView {
  pub id: Uuid,
  pub title: String,
  pub game_id: String,
  pub notes: String,
  pub client_secret: String,
}

impl ReviewIntentView {
  pub fn from(review: ReviewIntent, payment_intent: stripe::PaymentIntent) -> Self {
    ReviewIntentView {
      id: review.id,
      title: review.title,
      game_id: review.game_id,
      notes: review.notes,
      client_secret: *payment_intent.client_secret.unwrap(),
    }
  }
}
