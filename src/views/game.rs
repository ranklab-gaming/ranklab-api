use crate::data_types::SkillLevel;
use crate::models::Game;
use schemars::JsonSchema;
use serde::Serialize;

#[derive(Serialize, JsonSchema)]
#[serde(rename = "Game")]
pub struct GameView {
  name: String,
  id: String,
  skill_levels: Vec<SkillLevel>,
  min_coach_skill_level: SkillLevel,
}

impl From<&Box<dyn Game>> for GameView {
  fn from(game: &Box<dyn Game>) -> Self {
    GameView {
      name: game.name().to_owned(),
      id: game.id().to_owned(),
      skill_levels: game.skill_levels().to_owned(),
      min_coach_skill_level: game.min_coach_skill_level().to_owned(),
    }
  }
}
