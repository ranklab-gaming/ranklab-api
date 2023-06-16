use crate::data_types::MediaState;
use crate::schema::avatars;
use derive_builder::Builder;
use diesel::dsl::{And, Eq, Filter, FindBy};
use diesel::prelude::*;
use schemars::JsonSchema;
use serde::Serialize;
use uuid::Uuid;

#[derive(Builder, Queryable, Identifiable, Clone, Serialize, JsonSchema)]
#[builder(
  derive(AsChangeset, Insertable),
  pattern = "owned",
  name = "AvatarChangeset"
)]
#[builder_struct_attr(diesel(table_name = avatars))]
pub struct Avatar {
  pub id: Uuid,
  pub image_key: String,
  pub processed_image_key: Option<String>,
  pub created_at: chrono::NaiveDateTime,
  pub updated_at: chrono::NaiveDateTime,
  pub state: MediaState,
  pub user_id: Uuid,
}

impl Avatar {
  pub fn find_by_image_key(image_key: &str) -> FindBy<avatars::table, avatars::image_key, String> {
    avatars::table.filter(avatars::image_key.eq(image_key.to_string()))
  }

  pub fn find_processed_by_id(
    id: &Uuid,
  ) -> Filter<avatars::table, And<Eq<avatars::id, Uuid>, Eq<avatars::state, MediaState>>> {
    avatars::table.filter(
      avatars::id
        .eq(*id)
        .and(avatars::state.eq(MediaState::Processed)),
    )
  }

  pub fn find_for_user(
    user_id: &Uuid,
    id: &Uuid,
  ) -> Filter<avatars::table, And<Eq<avatars::id, Uuid>, Eq<avatars::user_id, Uuid>>> {
    avatars::table.filter(avatars::id.eq(*id).and(avatars::user_id.eq(*user_id)))
  }
}
