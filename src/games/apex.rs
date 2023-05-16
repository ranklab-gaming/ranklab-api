use crate::models::{Game, SkillLevel};

pub fn apex() -> Game {
  Game {
    skill_levels: SkillLevel::new_vec(vec![
      "Rookie", "Bronze", "Silver", "Gold", "Platinum", "Diamond", "Master",
    ]),
    name: "Apex Legends".to_string(),
    id: "apex".to_string(),
  }
}
