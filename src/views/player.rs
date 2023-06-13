use crate::intercom;
use crate::models::Avatar;
use crate::{config::Config, models::Player};
use schemars::JsonSchema;
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize, JsonSchema)]
#[serde(rename = "Player")]
pub struct PlayerView {
  pub id: Uuid,
  pub name: String,
  pub email: String,
  pub game_id: String,
  pub skill_level: i16,
  pub emails_enabled: bool,
  pub intercom_hash: Option<String>,
  pub avatar_image_key: Option<String>,
}

impl From<Player> for PlayerView {
  fn from(player: Player) -> Self {
    PlayerView::new(player, None, None)
  }
}

impl PlayerView {
  pub fn new(player: Player, config: Option<&Config>, avatar: Option<Avatar>) -> Self {
    let intercom_hash =
      config.and_then(|config| intercom::generate_user_hash(&player.email, config));

    PlayerView {
      id: player.id,
      name: player.name,
      email: player.email,
      game_id: player.game_id,
      skill_level: player.skill_level,
      emails_enabled: player.emails_enabled,
      intercom_hash,
      avatar_image_key: avatar.and_then(|avatar| avatar.processed_image_key),
    }
  }
}
