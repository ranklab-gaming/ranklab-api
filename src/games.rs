pub mod apex;
pub mod chess;
pub mod csgo;
pub mod dota2;
pub mod gym;
pub mod hearthstone;
pub mod lol;
pub mod overwatch;
pub mod r6s;
pub mod test;
pub mod valorant;

use crate::models::Game;
use lazy_static::lazy_static;
use validator::ValidationError;

lazy_static! {
  static ref GAMES: Vec<Game> = vec![
    test::test(),
    apex::apex(),
    chess::chess(),
    csgo::csgo(),
    dota2::dota2(),
    hearthstone::hearthstone(),
    lol::lol(),
    overwatch::overwatch(),
    r6s::r6s(),
    valorant::valorant(),
    gym::gym(),
  ];
}

pub fn all() -> &'static Vec<Game> {
  &GAMES
}

pub fn find(id: &str) -> Option<&'static Game> {
  all().iter().find(|g| g.id == id)
}

pub fn filter(ids: Vec<&str>) -> Vec<&'static Game> {
  all()
    .iter()
    .filter(|g| ids.contains(&g.id.as_str()))
    .collect()
}

pub fn validate_id(id: &str) -> Result<(), ValidationError> {
  match crate::games::find(id) {
    Some(_) => Ok(()),
    None => Err(ValidationError::new("Invalid game ID")),
  }
}
