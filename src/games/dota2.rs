use crate::models::{Game, SkillLevel};

pub fn dota2() -> Game {
  Game {
    id: super::GameId::Dota2,
    skill_levels: SkillLevel::new_vec(vec![
      "Herald", "Guardian", "Crusader", "Archon", "Legend", "Ancient", "Divine", "Immortal",
    ]),
    name: "Dota 2".to_string(),
  }
}
