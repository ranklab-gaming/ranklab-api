use crate::models::{Game, SkillLevel};

pub fn hearthstone() -> Game {
  Game {
    skill_levels: SkillLevel::new_vec(vec!["Bronze", "Silver", "Gold", "Platinum", "Diamond"]),
    name: "Hearthstone".to_string(),
    id: "hearthstone".to_string(),
  }
}
