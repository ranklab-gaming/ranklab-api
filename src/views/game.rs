use crate::models::{Game, SkillLevel};
use schemars::JsonSchema;
use serde::Serialize;

#[derive(Serialize, JsonSchema)]
#[serde(rename = "Game")]
pub struct GameView {
  name: String,
  id: String,
  skill_levels: Vec<SkillLevel>,
}

impl From<&Game> for GameView {
  fn from(game: &Game) -> Self {
    GameView {
      name: game.name.to_owned(),
      id: game.id.to_owned(),
      skill_levels: game.skill_levels.to_owned(),
    }
  }
}
