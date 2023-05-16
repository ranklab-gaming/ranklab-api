use crate::models::{Game, SkillLevel};

pub fn r6s() -> Game {
  Game {
    skill_levels: SkillLevel::new_vec(vec![
      "Copper", "Bronze", "Silver", "Gold", "Platinum", "Emerald", "Diamond",
    ]),
    name: "R6S".to_string(),
    id: "r6s".to_string(),
  }
}
