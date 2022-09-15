pub mod overwatch;
pub mod valorant;

use crate::models::Game;
use lazy_static::lazy_static;
use validator::ValidationError;

lazy_static! {
  static ref GAMES: Vec<Box<dyn Game>> = vec![
    Box::new(overwatch::Overwatch::new()),
    Box::new(valorant::Valorant::new())
  ];
}

pub fn all() -> &'static Vec<Box<dyn Game>> {
  &GAMES
}

pub fn find(id: &str) -> Option<&'static Box<dyn Game>> {
  all().iter().find(|g| g.id() == id)
}

pub fn validate_id(id: &str) -> Result<(), ValidationError> {
  match crate::games::find(id) {
    Some(_) => Ok(()),
    None => Err(ValidationError::new("Invalid game ID")),
  }
}

pub fn validate_ids(ids: &Vec<String>) -> Result<(), ValidationError> {
  for id in ids {
    validate_id(id)?;
  }

  Ok(())
}
