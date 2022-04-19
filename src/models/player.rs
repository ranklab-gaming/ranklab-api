use crate::data_types::UserGame;
use crate::schema::players;
use derive_builder::Builder;
use diesel::dsl::FindBy;
use diesel::prelude::*;
use uuid::Uuid;

#[derive(Builder, Queryable, Identifiable)]
#[builder(
  derive(AsChangeset, Insertable),
  pattern = "owned",
  name = "PlayerChangeset"
)]
#[builder_struct_attr(table_name = "players")]
pub struct Player {
  pub auth0_id: String,
  pub email: String,
  pub games: Vec<UserGame>,
  pub id: Uuid,
  pub name: String,
  pub stripe_customer_id: Option<String>,
}

impl Player {
  pub fn find_by_auth0_id(auth0_id: String) -> FindBy<players::table, players::auth0_id, String> {
    players::table.filter(players::auth0_id.eq(auth0_id))
  }
}
