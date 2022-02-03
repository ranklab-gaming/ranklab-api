use crate::models::User;
use crate::views::{CoachView, PlayerView};
use schemars::JsonSchema;
use serde::Serialize;

#[derive(Serialize, JsonSchema)]
#[serde(rename = "User", tag = "type")]
pub enum UserView {
  CoachView(CoachView),
  PlayerView(PlayerView),
}

impl From<User> for UserView {
  fn from(user: User) -> Self {
    match user {
      User::Coach(coach) => UserView::CoachView(coach.into()),
      User::Player(player) => UserView::PlayerView(player.into()),
    }
  }
}
