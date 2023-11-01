use crate::models::{Game, SkillLevel};

pub fn cs2() -> Game {
  Game {
    id: super::GameId::Cs2,
    skill_levels: SkillLevel::new_vec(vec![
      "Silver",
      "Silver Elite",
      "Silver Elite Master",
      "Gold Nova",
      "Gold Nova Master",
      "Master Guardian",
      "Master Guardian Elite",
      "Distinguished Master Guardian",
      "Legendary Eagle",
      "Legendary Eagle Master",
      "Supreme Master First Class",
      "Global Elite",
    ]),
    name: "Counter-Strike 2".to_string(),
  }
}
