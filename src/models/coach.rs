use crate::data_types::UserGame;
use crate::schema::coaches;
use derive_builder::Builder;
use diesel::helper_types::FindBy;
use diesel::prelude::*;
use uuid::Uuid;

#[derive(Builder, Queryable, Identifiable)]
#[table_name = "coaches"]
#[builder(derive(AsChangeset), pattern = "owned", name = "CoachChangeset")]
#[builder_struct_attr(table_name = "coaches")]
pub struct Coach {
  pub auth0_id: String,
  pub bio: String,
  pub country: String,
  pub email: String,
  pub games: Vec<UserGame>,
  pub id: Uuid,
  pub name: String,
  pub stripe_account_id: Option<String>,
  pub stripe_details_submitted: bool,
  pub stripe_payouts_enabled: bool,
}

impl Coach {
  pub fn find_by_stripe_account_id<T: ToString>(
    stripe_account_id: T,
  ) -> FindBy<coaches::table, coaches::stripe_account_id, Option<String>> {
    coaches::table.filter(coaches::stripe_account_id.eq(Some(stripe_account_id.to_string())))
  }

  pub fn all(conn: &PgConnection) -> QueryResult<Vec<Self>> {
    coaches::table.load(conn)
  }
}
