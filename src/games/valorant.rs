use crate::models::{Game, SkillLevel};

pub fn valorant() -> Game {
  Game {
    skill_levels: SkillLevel::new_vec(vec![
      "Iron", "Bronze", "Silver", "Gold", "Platinum", "Diamond", "Immortal", "Radiant",
    ]),
    name: "Valorant".to_string(),
    id: "valorant".to_string(),
  }
}
