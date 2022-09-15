use crate::data_types::SkillLevel;
use crate::models::Game;

pub struct Overwatch {
  skill_levels: Vec<SkillLevel>,
  name: String,
  id: String,
}

impl Overwatch {
  pub fn new() -> Self {
    Self {
      skill_levels: SkillLevel::new_vec(vec![
        "Bronze",
        "Silver",
        "Gold",
        "Platinum",
        "Diamond",
        "Masters",
        "Grandmaster",
      ]),
      name: "Overwatch".to_string(),
      id: "overwatch".to_string(),
    }
  }
}

impl Game for Overwatch {
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
