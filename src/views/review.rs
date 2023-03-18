use crate::data_types::ReviewState;
use crate::models::{Coach, Recording, Review};
use schemars::JsonSchema;
use serde::Serialize;
use stripe::PaymentIntent;
use uuid::Uuid;

use super::{CoachView, RecordingView};

#[derive(Serialize, JsonSchema)]
#[serde(rename = "Review")]
pub struct ReviewView {
  pub id: Uuid,
  pub player_id: Uuid,
  pub coach_id: Uuid,
  pub recording_id: Uuid,
  pub recording: Option<RecordingView>,
  pub notes: String,
  pub state: ReviewState,
  pub created_at: chrono::NaiveDateTime,
  pub stripe_client_secret: Option<String>,
  pub coach: Option<CoachView>,
}

pub struct ReviewViewOptions {
  pub payment_intent: Option<PaymentIntent>,
  pub coach: Option<Coach>,
  pub recording: Option<Recording>,
}

impl ReviewView {
  pub fn new(review: Review, options: ReviewViewOptions) -> Self {
    ReviewView {
      id: review.id,
      player_id: review.player_id,
      coach_id: review.coach_id,
      recording_id: review.recording_id,
      notes: review.notes,
      state: review.state,
      created_at: review.created_at,
      recording: options
        .recording
        .map(|recording| RecordingView::new(recording, None)),
      stripe_client_secret: options
        .payment_intent
        .map(|payment_intent| payment_intent.client_secret.unwrap()),
      coach: options.coach.map(|coach| coach.into()),
    }
  }
}

impl From<Review> for ReviewView {
  fn from(review: Review) -> Self {
    ReviewView::new(
      review,
      ReviewViewOptions {
        payment_intent: None,
        coach: None,
        recording: None,
      },
    )
  }
}
