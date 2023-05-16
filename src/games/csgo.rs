use crate::models::{Game, SkillLevel};

pub fn csgo() -> Game {
  Game {
    skill_levels: SkillLevel::new_vec(vec![
      "Silver",
      "Gold Nova",
      "Master Guardian",
      "Legendary Eagle",
      "Supreme Master First Class",
    ]),
    name: "CS:GO".to_string(),
    id: "csgo".to_string(),
  }
}
