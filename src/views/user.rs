use crate::config::Config;
use crate::intercom;
use crate::models::{Avatar, User};
use schemars::JsonSchema;
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize, JsonSchema)]
#[serde(rename = "User")]
pub struct UserView {
  pub id: Uuid,
  pub name: String,
  pub email: String,
  pub emails_enabled: bool,
  pub intercom_hash: Option<String>,
  pub avatar_id: Option<Uuid>,
  pub avatar_image_key: Option<String>,
}

impl From<User> for UserView {
  fn from(user: User) -> Self {
    UserView::new(user, None, None)
  }
}

impl UserView {
  pub fn new(user: User, config: Option<&Config>, avatar: Option<Avatar>) -> Self {
    let intercom_hash = config.and_then(|config| intercom::generate_user_hash(&user.email, config));

    UserView {
      id: user.id,
      name: user.name,
      email: user.email,
      emails_enabled: user.emails_enabled,
      intercom_hash,
      avatar_id: user.avatar_id,
      avatar_image_key: avatar.and_then(|avatar| avatar.processed_image_key),
    }
  }
}
