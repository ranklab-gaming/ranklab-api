use crate::guards::auth::UserType;
use crate::schema::one_time_tokens;
use derive_builder::Builder;
use diesel::dsl::{Find, FindBy};
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::sql_types::Bool;
use uuid::Uuid;

#[derive(Builder, Queryable, Identifiable, Clone)]
#[builder(
  derive(AsChangeset, Insertable),
  pattern = "owned",
  name = "OneTimeTokenChangeset"
)]
#[builder_struct_attr(diesel(table_name = one_time_tokens))]
pub struct OneTimeToken {
  pub id: Uuid,
  pub value: String,
  pub player_id: Option<Uuid>,
  pub coach_id: Option<Uuid>,
  pub scope: String,
  pub used_at: Option<chrono::NaiveDateTime>,
  pub updated_at: chrono::NaiveDateTime,
  pub created_at: chrono::NaiveDateTime,
}

impl OneTimeToken {
  pub fn find_by_value(
    value: &str,
    user_type: UserType,
    scope: &str,
  ) -> Filter<
    one_time_tokens::table,
    And<
      Or<
        Eq<one_time_tokens::coach_id, Option<Uuid>>,
        Eq<one_time_tokens::state, one_time_tokenstate>,
      >,
      Eq<one_time_tokens::recording_id, Uuid>,
    >,
  > {
    let user_type_expr: Box<dyn BoxableExpression<one_time_tokens::table, Pg, SqlType = Bool>> =
      match user_type {
        UserType::Player => Box::new(one_time_tokens::player_id.is_not_null()),
        UserType::Coach => Box::new(one_time_tokens::coach_id.is_not_null()),
      };

    one_time_tokens::table.filter(
      one_time_tokens::value
        .eq(value.to_string())
        .and(user_type_expr)
        .and(one_time_tokens::scope.eq(scope.to_string())),
    )
  }
}
