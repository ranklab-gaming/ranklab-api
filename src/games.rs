pub mod hearthstone;
pub mod overwatch;
pub mod r6s;
pub mod valorant;

use crate::models::Game;
use lazy_static::lazy_static;
use validator::ValidationError;

lazy_static! {
  static ref GAMES: Vec<Game> = vec![
    overwatch::overwatch(),
    valorant::valorant(),
    hearthstone::hearthstone(),
    r6s::r6s(),
  ];
}

pub fn all() -> &'static Vec<Game> {
  &GAMES
}

pub fn find(id: &str) -> Option<&'static Game> {
  all().iter().find(|g| g.id == id)
}

pub fn validate_id(id: &str) -> Result<(), ValidationError> {
  match crate::games::find(id) {
    Some(_) => Ok(()),
    None => Err(ValidationError::new("Invalid game ID")),
  }
}
