use crate::schema::coaches;
use derive_builder::Builder;
use diesel::helper_types::{And, Eq, EqAny, Filter, Find, FindBy};
use diesel::prelude::*;
use uuid::Uuid;

#[derive(Builder, Queryable, Identifiable, Clone)]
#[diesel(table_name = coaches)]
#[builder(
  derive(AsChangeset, Insertable),
  pattern = "owned",
  name = "CoachChangeset"
)]
#[builder_struct_attr(diesel(table_name = coaches))]
pub struct Coach {
  pub bio: String,
  pub country: String,
  pub created_at: chrono::NaiveDateTime,
  pub email: String,
  pub game_id: String,
  pub id: Uuid,
  pub name: String,
  pub password: Option<String>,
  pub price: i32,
  pub stripe_account_id: String,
  pub stripe_details_submitted: bool,
  pub stripe_payouts_enabled: bool,
  pub updated_at: chrono::NaiveDateTime,
  pub emails_enabled: bool,
  pub slug: String,
  pub avatar_id: Option<Uuid>,
  pub approved: bool,
}

impl Coach {
  pub fn find_by_stripe_account_id<T: ToString>(
    stripe_account_id: &T,
  ) -> FindBy<coaches::table, coaches::stripe_account_id, String> {
    coaches::table.filter(coaches::stripe_account_id.eq(stripe_account_id.to_string()))
  }

  pub fn find_by_id(id: &Uuid) -> Find<coaches::table, Uuid> {
    coaches::table.find(*id)
  }

  pub fn find_by_email(email: &str) -> FindBy<coaches::table, coaches::email, String> {
    coaches::table.filter(coaches::email.eq(email.to_string()))
  }

  pub fn find_by_slug(slug: &str) -> FindBy<coaches::table, coaches::slug, String> {
    coaches::table.filter(coaches::slug.eq(slug.to_string()))
  }

  pub fn filter_by_ids(ids: Vec<Uuid>) -> Filter<coaches::table, EqAny<coaches::id, Vec<Uuid>>> {
    coaches::table.filter(coaches::id.eq_any(ids))
  }

  #[allow(clippy::type_complexity)]
  pub fn filter_by_game_id(
    game_id: &str,
  ) -> Filter<coaches::table, And<Eq<coaches::game_id, String>, Eq<coaches::approved, bool>>> {
    coaches::table.filter(
      coaches::game_id
        .eq(game_id.to_string())
        .and(coaches::approved.eq(true)),
    )
  }

  #[allow(clippy::type_complexity)]
  pub fn find_for_game_id(
    coach_id: &Uuid,
    game_id: &str,
  ) -> Filter<
    coaches::table,
    And<And<Eq<coaches::id, Uuid>, Eq<coaches::game_id, String>>, Eq<coaches::approved, bool>>,
  > {
    coaches::table.filter(
      coaches::id
        .eq(*coach_id)
        .and(coaches::game_id.eq(game_id.to_string()))
        .and(coaches::approved.eq(true)),
    )
  }
}
