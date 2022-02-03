use crate::data_types::SkillLevel;

pub trait Game: Send + Sync + 'static {
  fn skill_levels(&self) -> &Vec<SkillLevel>;
  fn name(&self) -> &str;
  fn id(&self) -> &str;
  fn min_coach_skill_level(&self) -> &SkillLevel;
}
