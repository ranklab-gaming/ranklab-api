use crate::schema::followings;
use crate::schema::users;
use derive_builder::Builder;
use diesel::dsl::{Find, FindBy};
use diesel::helper_types::On;
use diesel::helper_types::Select;
use diesel::helper_types::{Eq, EqAny, Filter, InnerJoin};
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
  pub id: Uuid,
  pub name: String,
  pub password: Option<String>,
  pub updated_at: chrono::NaiveDateTime,
  pub emails_enabled: bool,
  pub avatar_id: Option<Uuid>,
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

  pub fn filter_for_digest() -> Select<
    Filter<
      InnerJoin<users::table, On<followings::table, Eq<users::id, followings::user_id>>>,
      Eq<users::emails_enabled, bool>,
    >,
    (
      <users::table as diesel::Table>::AllColumns,
      followings::game_id,
    ),
  > {
    users::table
      .inner_join(followings::table.on(users::id.eq(followings::user_id)))
      .filter(users::emails_enabled.eq(true))
      .select((users::all_columns, followings::game_id))
  }

  pub fn all() -> users::table {
    users::table
  }
}
