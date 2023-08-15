use crate::schema::users;
use derive_builder::Builder;
use diesel::dsl::{Find, FindBy};
use diesel::helper_types::{EqAny, Filter};
use diesel::prelude::*;
use uuid::Uuid;

#[derive(Builder, Queryable, Identifiable, Clone)]
#[builder(
  derive(AsChangeset, Insertable),
  pattern = "owned",
  name = "UserChangeset"
)]
#[builder_struct_attr(diesel(table_name = users))]
pub struct User {
  pub created_at: chrono::NaiveDateTime,
  pub email: String,
  pub game_id: String,
  pub id: Uuid,
  pub name: String,
  pub password: Option<String>,
  pub updated_at: chrono::NaiveDateTime,
  pub emails_enabled: bool,
  pub avatar_id: Option<Uuid>,
  pub skill_level: i16,
  pub digest_notified_at: chrono::NaiveDateTime,
}

impl User {
  pub fn find_by_id(id: &Uuid) -> Find<users::table, Uuid> {
    users::table.find(*id)
  }

  pub fn find_by_email(email: &str) -> FindBy<users::table, users::email, String> {
    users::table.filter(users::email.eq(email.to_string()))
  }

  pub fn filter_by_ids(ids: Vec<Uuid>) -> Filter<users::table, EqAny<users::id, Vec<Uuid>>> {
    users::table.filter(users::id.eq_any(ids))
  }

  pub fn all() -> users::table {
    users::table
  }
}
