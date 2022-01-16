use crate::models::{Game, SkillLevel};

pub struct Overwatch;

impl Game for Overwatch {
  fn skill_levels(&self) -> Vec<SkillLevel> {
    SkillLevel::new_vec(vec![
      "Bronze",
      "Silver",
      "Gold",
      "Platinum",
      "Diamond",
      "Masters",
      "Grandmaster",
    ])
  }

  fn name(&self) -> String {
    "Overwatch".to_string()
  }

  fn id(&self) -> String {
    "overwatch".to_string()
  }
}
