use crate::games::GameId;
use schemars::JsonSchema;
use serde::Serialize;

#[derive(Serialize, JsonSchema, Clone)]
pub struct SkillLevel {
  pub name: String,
  pub value: u8,
}

impl SkillLevel {
  pub fn new_vec(skill_levels: Vec<&str>) -> Vec<Self> {
    skill_levels
      .iter()
      .enumerate()
      .map(|(value, &name)| Self {
        name: name.to_owned(),
        value: value as u8,
      })
      .collect()
  }
}

pub struct Game {
  pub id: GameId,
  pub skill_levels: Vec<SkillLevel>,
  pub name: String,
}
