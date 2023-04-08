use crate::{data_types::RecordingState, models::Recording};
use schemars::JsonSchema;
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize, JsonSchema)]
#[serde(rename = "Recording")]
pub struct RecordingView {
  pub id: Uuid,
  pub player_id: Uuid,
  pub video_key: Option<String>,
  pub thumbnail_key: Option<String>,
  pub upload_url: Option<String>,
  pub mime_type: String,
  pub created_at: chrono::NaiveDateTime,
  pub updated_at: chrono::NaiveDateTime,
  pub game_id: String,
  pub title: String,
  pub skill_level: i16,
  pub state: RecordingState,
}

impl From<Recording> for RecordingView {
  fn from(recording: Recording) -> Self {
    RecordingView::new(recording, None)
  }
}

impl RecordingView {
  pub fn new(recording: Recording, upload_url: Option<String>) -> Self {
    RecordingView {
      id: recording.id,
      player_id: recording.player_id,
      video_key: recording.processed_video_key,
      upload_url,
      mime_type: "video/mp4".to_string(),
      created_at: recording.created_at,
      updated_at: recording.updated_at,
      game_id: recording.game_id,
      title: recording.title,
      skill_level: recording.skill_level,
      state: recording.state,
      thumbnail_key: recording.thumbnail_key,
    }
  }
}
