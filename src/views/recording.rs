use crate::data_types::MediaState;
use crate::models::{Recording, User};
use schemars::JsonSchema;
use serde::Serialize;
use uuid::Uuid;

use super::UserView;

#[derive(Serialize, JsonSchema)]
#[serde(rename = "Recording")]
pub struct RecordingView {
  pub id: Uuid,
  pub user_id: Uuid,
  pub video_key: Option<String>,
  pub thumbnail_key: Option<String>,
  pub upload_url: Option<String>,
  pub created_at: chrono::NaiveDateTime,
  pub updated_at: chrono::NaiveDateTime,
  pub game_id: String,
  pub title: String,
  pub skill_level: i16,
  pub state: MediaState,
  pub metadata: Option<serde_json::Value>,
  pub instance_id: Option<String>,
  pub notes: String,
  pub user: Option<UserView>,
  pub notes_text: String,
  pub comment_count: i64,
}

impl From<Recording> for RecordingView {
  fn from(recording: Recording) -> Self {
    RecordingView::new(recording, None, None, None, None)
  }
}

impl RecordingView {
  pub fn new(
    recording: Recording,
    upload_url: Option<String>,
    instance_id: Option<String>,
    user: Option<User>,
    comment_count: Option<i64>,
  ) -> Self {
    let notes_text = html2text::from_read(recording.notes.as_bytes(), 100);

    RecordingView {
      id: recording.id,
      user_id: recording.user_id,
      video_key: recording.processed_video_key,
      upload_url,
      created_at: recording.created_at,
      updated_at: recording.updated_at,
      game_id: recording.game_id,
      title: recording.title,
      skill_level: recording.skill_level,
      state: recording.state,
      thumbnail_key: recording.thumbnail_key,
      metadata: recording.metadata,
      instance_id,
      notes: recording.notes,
      notes_text,
      user: user.map(UserView::from),
      comment_count: comment_count.unwrap_or(0),
    }
  }
}
