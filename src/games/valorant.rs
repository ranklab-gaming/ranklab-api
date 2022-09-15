use crate::data_types::SkillLevel;
use crate::models::Game;

pub struct Valorant {
  skill_levels: Vec<SkillLevel>,
  name: String,
  id: String,
}

impl Valorant {
  pub fn new() -> Self {
    Self {
      skill_levels: SkillLevel::new_vec(vec![
        "Iron", "Bronze", "Silver", "Gold", "Platinum", "Diamond", "Immortal", "Radiant",
      ]),
      name: "Valorant".to_string(),
      id: "valorant".to_string(),
    }
  }
}

impl Game for Valorant {
  fn skill_levels(&self) -> &Vec<SkillLevel> {
    &self.skill_levels
  }

  fn name(&self) -> &str {
    &self.name
  }

  fn id(&self) -> &str {
    &self.id
  }
}
