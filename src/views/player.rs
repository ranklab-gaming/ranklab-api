use crate::data_types::UserGame;
use crate::models::Player;
use schemars::JsonSchema;
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize, JsonSchema)]
#[serde(rename = "Player")]
pub struct PlayerView {
  pub id: Uuid,
  pub auth0_id: String,
  pub name: String,
  pub email: String,
  pub games: Vec<UserGame>,
}

impl From<Player> for PlayerView {
  fn from(player: Player) -> Self {
    PlayerView {
      id: player.id,
      auth0_id: player.auth0_id,
      name: player.name,
      email: player.email,
      games: player.games.into_iter().map(|game| game.unwrap()).collect(),
    }
  }
}
