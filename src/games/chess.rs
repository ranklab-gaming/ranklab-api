use crate::models::{Game, SkillLevel};

pub fn chess() -> Game {
  Game {
    skill_levels: SkillLevel::new_vec(vec!["Beginner", "Intermediate", "Advanced", "Master"]),
    name: "Chess".to_string(),
    id: "chess".to_string(),
  }
}
