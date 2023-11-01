pub mod apex;
pub mod cs2;
pub mod dota2;
pub mod lol;
pub mod overwatch;
pub mod valorant;
use crate::models::Game;
use lazy_static::lazy_static;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(
  Debug, PartialEq, Deserialize, Serialize, Eq, Hash, Clone, JsonSchema, Copy, FromFormField,
)]
#[serde(rename_all = "snake_case")]
pub enum GameId {
  Overwatch,
  Apex,
  Cs2,
  Dota2,
  Lol,
  Valorant,
}

impl ToString for GameId {
  fn to_string(&self) -> String {
    serde_plain::to_string(&self).unwrap()
  }
}

lazy_static! {
  static ref GAMES: Vec<Game> = vec![
    overwatch::overwatch(),
    apex::apex(),
    cs2::cs2(),
    dota2::dota2(),
    lol::lol(),
    valorant::valorant(),
  ];
}

pub fn all() -> &'static Vec<Game> {
  &GAMES
}

pub fn find(id: &str) -> Option<&'static Game> {
  all().iter().find(|g| g.id.to_string() == id)
}
