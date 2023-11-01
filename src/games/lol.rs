use crate::models::{Game, SkillLevel};

pub fn lol() -> Game {
  Game {
    id: super::GameId::Lol,
    skill_levels: SkillLevel::new_vec(vec![
      "Iron",
      "Bronze",
      "Silver",
      "Gold",
      "Platinum",
      "Emerald",
      "Diamond",
      "Master",
      "Grandmaster",
      "Challenger",
    ]),
    name: "League of Legends".to_string(),
  }
}
