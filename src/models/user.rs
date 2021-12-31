use crate::models::{Coach, Player};
use schemars::JsonSchema;
use serde::Serialize;

#[derive(Serialize, JsonSchema)]
#[serde(tag = "type")]
pub enum User {
  Coach(Coach),
  Player(Player),
}
