use crate::models::{Recording, Review};
use schemars::JsonSchema;
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize, JsonSchema)]
#[serde(rename = "Recording")]
pub struct RecordingView {
  pub id: Uuid,
  pub player_id: Uuid,
  pub video_key: String,
  pub upload_url: Option<String>,
  pub uploaded: bool,
  pub mime_type: String,
  pub created_at: chrono::NaiveDateTime,
  pub updated_at: chrono::NaiveDateTime,
  pub review_title: Option<String>,
  pub review_id: Option<Uuid>,
}

impl RecordingView {
  pub fn new(recording: Recording, review: Option<&Review>, upload_url: Option<String>) -> Self {
    RecordingView {
      id: recording.id,
      player_id: recording.player_id,
      video_key: recording.video_key,
      upload_url,
      uploaded: recording.uploaded,
      mime_type: recording.mime_type,
      created_at: recording.created_at,
      updated_at: recording.updated_at,
      review_title: review.map(|review| review.title.clone()),
      review_id: review.map(|review| review.id),
    }
  }
}
