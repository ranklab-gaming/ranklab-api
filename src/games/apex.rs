use crate::models::{Game, SkillLevel};

pub fn apex() -> Game {
  Game {
    id: super::GameId::Apex,
    skill_levels: SkillLevel::new_vec(vec![
      "Bronze", "Silver", "Gold", "Platinum", "Diamond", "Master", "Predator",
    ]),
    name: "Apex Legends".to_string(),
  }
}
