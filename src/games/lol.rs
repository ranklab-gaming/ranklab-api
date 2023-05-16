use crate::models::{Game, SkillLevel};

pub fn lol() -> Game {
  Game {
    skill_levels: SkillLevel::new_vec(vec![
      "Iron",
      "Bronze",
      "Silver",
      "Gold",
      "Platinum",
      "Diamond",
      "Master",
      "Grandmaster",
    ]),
    name: "League of Legends".to_string(),
    id: "lol".to_string(),
  }
}
