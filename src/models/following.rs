use crate::schema::followings;
use derive_builder::Builder;
use diesel::dsl::{Eq, Filter};
use diesel::prelude::*;
use uuid::Uuid;

#[derive(Builder, Queryable, Identifiable, Clone)]
#[builder(
  derive(AsChangeset, Insertable),
  pattern = "owned",
  name = "FollowingChangeset"
)]
#[builder_struct_attr(diesel(table_name = followings))]
pub struct Following {
  pub id: Uuid,
  pub user_id: Uuid,
  pub game_id: String,
  pub created_at: chrono::NaiveDateTime,
  pub updated_at: chrono::NaiveDateTime,
}

#[allow(clippy::type_complexity)]
impl Following {
  pub fn filter_for_user(
    user_id: &Uuid,
  ) -> Filter<followings::table, Eq<followings::user_id, Uuid>> {
    followings::table.filter(followings::user_id.eq(*user_id))
  }

  pub fn find_for_user_and_game(
    user_id: &Uuid,
    game_id: &str,
  ) -> Filter<
    Filter<followings::table, Eq<followings::user_id, Uuid>>,
    Eq<followings::game_id, String>,
  > {
    followings::table
      .filter(followings::user_id.eq(*user_id))
      .filter(followings::game_id.eq(game_id.to_owned()))
  }
}
