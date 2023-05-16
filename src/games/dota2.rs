use crate::models::{Game, SkillLevel};

pub fn dota2() -> Game {
  Game {
    skill_levels: SkillLevel::new_vec(vec![
      "Herald", "Guardian", "Crusader", "Archon", "Legend", "Ancient", "Divine",
    ]),
    name: "Dota 2".to_string(),
    id: "dota2".to_string(),
  }
}
