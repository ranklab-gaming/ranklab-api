use crate::models::Recording;
use schemars::JsonSchema;
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize, JsonSchema)]
#[serde(rename = "Recording")]
pub struct RecordingView {
  pub id: Uuid,
  pub player_id: Uuid,
  pub video_key: String,
  pub upload_url: String,
  pub uploaded: bool,
  pub mime_type: String,
  pub created_at: chrono::NaiveDateTime,
  pub updated_at: chrono::NaiveDateTime,
}

impl From<Recording> for RecordingView {
  fn from(recording: Recording) -> Self {
    RecordingView {
      id: recording.id,
      player_id: recording.player_id,
      video_key: recording.video_key,
      upload_url: recording.upload_url,
      uploaded: recording.uploaded,
      mime_type: recording.mime_type,
      created_at: recording.created_at,
      updated_at: recording.updated_at,
    }
  }
}
