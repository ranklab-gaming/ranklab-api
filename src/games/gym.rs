use crate::models::{Game, SkillLevel};

pub fn gym() -> Game {
  Game {
    skill_levels: SkillLevel::new_vec(vec!["Beginner", "Intermediate", "Advanced", "Expert"]),
    name: "Personal Training".to_string(),
    id: "gym".to_string(),
  }
}
