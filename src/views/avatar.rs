use crate::{data_types::MediaState, models::Avatar};
use schemars::JsonSchema;
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize, JsonSchema)]
#[serde(rename = "Avatar")]
pub struct AvatarView {
  pub id: Uuid,
  pub image_key: Option<String>,
  pub upload_url: Option<String>,
  pub created_at: chrono::NaiveDateTime,
  pub updated_at: chrono::NaiveDateTime,
  pub state: MediaState,
  pub instance_id: Option<String>,
}

impl From<Avatar> for AvatarView {
  fn from(avatar: Avatar) -> Self {
    AvatarView::new(avatar, None, None)
  }
}

impl AvatarView {
  pub fn new(avatar: Avatar, upload_url: Option<String>, instance_id: Option<String>) -> Self {
    AvatarView {
      id: avatar.id,
      image_key: avatar.processed_image_key,
      upload_url,
      created_at: avatar.created_at,
      updated_at: avatar.updated_at,
      state: avatar.state,
      instance_id,
    }
  }
}
