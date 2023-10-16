pub mod apex;
pub mod overwatch;
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
}

impl ToString for GameId {
  fn to_string(&self) -> String {
    serde_plain::to_string(&self).unwrap()
  }
}

lazy_static! {
  static ref GAMES: Vec<Game> = vec![overwatch::overwatch(), apex::apex()];
}

pub fn all() -> &'static Vec<Game> {
  &GAMES
}

pub fn find(id: &str) -> Option<&'static Game> {
  all().iter().find(|g| g.id.to_string() == id)
}
