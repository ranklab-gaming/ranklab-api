use crate::models::{Game, SkillLevel};

pub fn overwatch() -> Game {
  Game {
    id: super::GameId::Overwatch,
    skill_levels: SkillLevel::new_vec(vec![
      "Bronze",
      "Silver",
      "Gold",
      "Platinum",
      "Diamond",
      "Masters",
      "Grandmaster",
    ]),
    name: "Overwatch".to_string(),
  }
}
