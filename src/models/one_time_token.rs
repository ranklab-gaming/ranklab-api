use crate::schema::one_time_tokens;
use derive_builder::Builder;
use diesel::dsl::{And, Eq, Filter};
use diesel::helper_types::{IsNotNull, IsNull};
use diesel::prelude::*;
use uuid::Uuid;

#[derive(Builder, Queryable, Identifiable, Clone)]
#[builder(
  derive(AsChangeset, Insertable),
  pattern = "owned",
  name = "OneTimeTokenChangeset"
)]
#[builder_struct_attr(diesel(table_name = one_time_tokens))]
pub struct OneTimeToken {
  pub created_at: chrono::NaiveDateTime,
  pub id: Uuid,
  pub user_id: Option<Uuid>,
  pub updated_at: chrono::NaiveDateTime,
  pub used_at: Option<chrono::NaiveDateTime>,
  pub value: String,
  pub scope: String,
}

#[allow(clippy::type_complexity)]
impl OneTimeToken {
  pub fn find_by_value(
    value: &str,
    scope: &str,
  ) -> Filter<
    one_time_tokens::table,
    And<
      And<
        And<Eq<one_time_tokens::value, String>, Eq<one_time_tokens::scope, String>>,
        IsNotNull<one_time_tokens::user_id>,
      >,
      IsNull<one_time_tokens::used_at>,
    >,
  > {
    one_time_tokens::table.filter(
      one_time_tokens::value
        .eq(value.to_string())
        .and(one_time_tokens::scope.eq(scope.to_string()))
        .and(one_time_tokens::user_id.is_not_null())
        .and(one_time_tokens::used_at.is_null()),
    )
  }
}
