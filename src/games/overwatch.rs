use crate::games::{Game, SkillLevel};

pub struct Overwatch;

impl Game for Overwatch {
  fn skill_levels(&self) -> Vec<SkillLevel> {
    SkillLevel::new_vec(vec![
      ("Bronze".to_string(), 1),
      ("Silver".to_string(), 2),
      ("Gold".to_string(), 3),
      ("Platinum".to_string(), 4),
      ("Diamond".to_string(), 5),
      ("Masters".to_string(), 6),
      ("Grandmaster".to_string(), 7),
    ])
  }

  fn name(&self) -> String {
    "Overwatch".to_string()
  }

  fn id(&self) -> String {
    "overwatch".to_string()
  }
}
