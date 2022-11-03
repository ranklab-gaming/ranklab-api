use super::{Coach, Player};

pub enum Account {
  Player(Player),
  Coach(Coach),
}
