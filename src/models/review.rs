use crate::data_types::ReviewState;
use crate::schema::reviews;
use derive_builder::Builder;
use diesel::dsl::FindBy;
use diesel::prelude::*;
use uuid::Uuid;

#[derive(Builder, Queryable, Identifiable)]
#[builder(derive(AsChangeset), pattern = "owned", name = "ReviewChangeset")]
#[builder_struct_attr(table_name = "reviews")]
pub struct Review {
  pub coach_id: Option<Uuid>,
  pub game_id: String,
  pub id: Uuid,
  pub notes: String,
  pub player_id: Uuid,
  pub recording_id: Uuid,
  pub skill_level: i16,
  pub title: String,
  pub state: ReviewState,
  pub stripe_order_id: String,
}

impl Review {
  pub fn find_by_order_id<T: ToString>(
    order_id: T,
  ) -> FindBy<reviews::table, reviews::stripe_order_id, String> {
    reviews::table.filter(reviews::stripe_order_id.eq(order_id.to_string()))
  }
}
