use crate::data_types::MediaState;
use crate::models::Audio;
use schemars::JsonSchema;
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize, JsonSchema)]
#[serde(rename = "Audio")]
pub struct AudioView {
  pub id: Uuid,
  pub audio_key: Option<String>,
  pub upload_url: Option<String>,
  pub created_at: chrono::NaiveDateTime,
  pub updated_at: chrono::NaiveDateTime,
  pub state: MediaState,
  pub instance_id: Option<String>,
  pub transcript: Option<String>,
}

impl From<Audio> for AudioView {
  fn from(audio: Audio) -> Self {
    AudioView::new(audio, None, None)
  }
}

impl AudioView {
  pub fn new(audio: Audio, upload_url: Option<String>, instance_id: Option<String>) -> Self {
    AudioView {
      id: audio.id,
      audio_key: audio.processed_audio_key,
      upload_url,
      created_at: audio.created_at,
      updated_at: audio.updated_at,
      state: audio.state,
      instance_id,
      transcript: audio.transcript,
    }
  }
}
