use crate::schema::coach_invitations;
use derive_builder::Builder;
use diesel::helper_types::FindBy;
use diesel::prelude::*;
use uuid::Uuid;

#[derive(Builder, Queryable, Identifiable, Clone)]
#[builder(
  derive(AsChangeset, Insertable),
  pattern = "owned",
  name = "CoachInvitationChangeset"
)]
#[builder_struct_attr(diesel(table_name = coach_invitations))]
pub struct CoachInvitation {
  pub created_at: chrono::NaiveDateTime,
  pub id: Uuid,
  pub updated_at: chrono::NaiveDateTime,
  pub used_at: Option<chrono::NaiveDateTime>,
  pub value: String,
}

#[allow(clippy::type_complexity)]
impl CoachInvitation {
  pub fn find_by_value(
    value: &str,
  ) -> FindBy<coach_invitations::table, coach_invitations::value, String> {
    coach_invitations::table.filter(coach_invitations::value.eq(value.to_string()))
  }
}
