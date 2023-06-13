use crate::schema::players;
use derive_builder::Builder;
use diesel::dsl::{EqAny, Filter, Find, FindBy};
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
  pub game_id: String,
  pub id: Uuid,
  pub name: String,
  pub password: Option<String>,
  pub skill_level: i16,
  pub stripe_customer_id: String,
  pub updated_at: chrono::NaiveDateTime,
  pub emails_enabled: bool,
  pub avatar_id: Option<Uuid>,
}

impl Player {
  pub fn find_by_id(id: &Uuid) -> Find<players::table, Uuid> {
    players::table.find(*id)
  }

  pub fn find_by_email(email: &str) -> FindBy<players::table, players::email, String> {
    players::table.filter(players::email.eq(email.to_string()))
  }

  pub fn filter_by_ids(ids: Vec<Uuid>) -> Filter<players::table, EqAny<players::id, Vec<Uuid>>> {
    players::table.filter(players::id.eq_any(ids))
  }
}
