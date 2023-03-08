use crate::models::Player;
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
}

impl From<Player> for PlayerView {
  fn from(player: Player) -> Self {
    PlayerView {
      id: player.id,
      name: player.name,
      email: player.email,
      game_id: player.game_id,
      skill_level: player.skill_level,
    }
  }
}
