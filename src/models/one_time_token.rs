use crate::guards::auth::UserType;
use crate::guards::DbConn;
use crate::schema::one_time_tokens;
use derive_builder::Builder;
use diesel::dsl::{And, Eq, Filter};
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::sql_types::Bool;
use uuid::Uuid;

use super::{Account, Coach, Player};

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

#[allow(clippy::type_complexity)]
impl OneTimeToken {
  pub fn find_by_value(
    value: &str,
    user_type: UserType,
    scope: &str,
  ) -> Filter<
    one_time_tokens::table,
    And<
      And<
        Eq<one_time_tokens::value, String>,
        Box<dyn BoxableExpression<one_time_tokens::table, Pg, SqlType = Bool>>,
      >,
      Eq<one_time_tokens::scope, String>,
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

  pub async fn account(&self, db_conn: &DbConn) -> Result<Account, diesel::result::Error> {
    match (self.player_id, self.coach_id) {
      (Some(player_id), None) => Ok(Account::Player(
        db_conn
          .run(move |conn| Player::find_by_id(&player_id).first(conn))
          .await?,
      )),
      (None, Some(coach_id)) => Ok(Account::Coach(
        db_conn
          .run(move |conn| Coach::find_by_id(&coach_id).first(conn))
          .await?,
      )),
      _ => Err(diesel::result::Error::NotFound),
    }
  }
}
