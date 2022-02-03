use crate::models::{Coach, Player};

pub enum User {
  Coach(Coach),
  Player(Player),
}
