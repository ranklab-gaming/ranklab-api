use crate::schema::coaches;
use derive_builder::Builder;
use diesel::helper_types::{Find, FindBy};
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
  pub email: String,
  pub name: String,
  pub bio: String,
  pub country: String,
  pub game_ids: Vec<Option<String>>,
  pub id: Uuid,
  pub password: String,
  pub stripe_account_id: Option<String>,
  pub stripe_details_submitted: bool,
  pub stripe_payouts_enabled: bool,
  pub updated_at: chrono::NaiveDateTime,
  pub created_at: chrono::NaiveDateTime,
}

impl Coach {
  pub fn find_by_stripe_account_id<T: ToString>(
    stripe_account_id: &T,
  ) -> FindBy<coaches::table, coaches::stripe_account_id, Option<String>> {
    coaches::table.filter(coaches::stripe_account_id.eq(Some(stripe_account_id.to_string())))
  }

  pub fn find_by_id(id: &Uuid) -> Find<coaches::table, Uuid> {
    coaches::table.find(*id)
  }

  pub fn find_by_email(email: &str) -> FindBy<coaches::table, coaches::email, String> {
    coaches::table.filter(coaches::email.eq(email.to_string()))
  }
}
