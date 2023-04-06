use crate::models::{Game, SkillLevel};

pub fn test() -> Game {
  Game {
    skill_levels: SkillLevel::new_vec(vec![
      "Very Low",
      "Low",
      "Medium",
      "High",
      "Very High",
      "Extreme",
    ]),
    name: "Test".to_string(),
    id: "test".to_string(),
  }
}
