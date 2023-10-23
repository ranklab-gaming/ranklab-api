use crate::games::GameId;
use crate::models::{Game, SkillLevel};
use schemars::JsonSchema;
use serde::Serialize;

#[derive(Serialize, JsonSchema)]
#[serde(rename = "Game")]
pub struct GameView {
  name: String,
  id: GameId,
  skill_levels: Vec<SkillLevel>,
  followed: bool,
}

impl From<&Game> for GameView {
  fn from(game: &Game) -> Self {
    GameView::new(game, false)
  }
}

impl GameView {
  pub fn new(game: &Game, followed: bool) -> Self {
    GameView {
      name: game.name.to_owned(),
      id: game.id,
      skill_levels: game.skill_levels.to_owned(),
      followed,
    }
  }
}
