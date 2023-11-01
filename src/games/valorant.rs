use crate::models::{Game, SkillLevel};

pub fn valorant() -> Game {
  Game {
    id: super::GameId::Valorant,
    skill_levels: SkillLevel::new_vec(vec![
      "Iron", "Bronze", "Silver", "Gold", "Platinum", "Diamond", "Immortal", "Radiant",
    ]),
    name: "Valorant".to_string(),
  }
}
