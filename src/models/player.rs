use crate::data_types::PlayerGame;
use crate::schema::players;
use derive_builder::Builder;
use diesel::dsl::{Find, FindBy};
use diesel::prelude::*;
use uuid::Uuid;

#[derive(Builder, Queryable, Identifiable, Clone)]
#[builder(
  derive(AsChangeset, Insertable),
  pattern = "owned",
  name = "PlayerChangeset"
)]
#[builder_struct_attr(diesel(table_name = players))]
pub struct Player {
  pub created_at: chrono::NaiveDateTime,
  pub email: String,
  pub games: Vec<Option<PlayerGame>>,
  pub id: Uuid,
  pub name: String,
  pub password: String,
  pub stripe_customer_id: Option<String>,
  pub updated_at: chrono::NaiveDateTime,
}

impl Player {
  pub fn find_by_id(id: &Uuid) -> Find<players::table, Uuid> {
    players::table.find(*id)
  }

  pub fn find_by_email(email: &str) -> FindBy<players::table, players::email, String> {
    players::table.filter(players::email.eq(email.to_string()))
  }
}
