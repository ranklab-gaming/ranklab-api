use schemars::JsonSchema;
use serde::Serialize;

#[derive(Serialize, JsonSchema)]
pub struct SkillLevel {
  name: String,
  value: u8,
}

impl SkillLevel {
  pub fn new_vec(skill_levels: Vec<&str>) -> Vec<Self> {
    skill_levels
      .iter()
      .enumerate()
      .map(|(value, &name)| Self {
        name: name.to_owned(),
        value: value as u8,
      })
      .collect()
  }
}
