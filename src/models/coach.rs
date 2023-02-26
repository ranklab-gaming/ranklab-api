use crate::schema::coaches;
use derive_builder::Builder;
use diesel::helper_types::{Filter, Find, FindBy, ILike};
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
  pub game_ids: Vec<Option<String>>,
  pub id: Uuid,
  pub name: String,
  pub password: String,
  pub stripe_account_id: String,
  pub stripe_details_submitted: bool,
  pub stripe_payouts_enabled: bool,
  pub stripe_product_id: String,
  pub updated_at: chrono::NaiveDateTime,
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

  pub fn find_by_query(query: &str) -> Filter<coaches::table, ILike<coaches::name, String>> {
    coaches::table.filter(coaches::name.ilike(format!("%{}%", query)))
  }
}
