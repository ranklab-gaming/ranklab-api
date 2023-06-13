use crate::data_types::MediaState;
use crate::schema::avatars;
use derive_builder::Builder;
use diesel::dsl::{And, Eq, EqAny, Filter, FindBy};
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
}

impl Avatar {
  pub fn find_by_image_key<T: ToString>(
    image_key: &T,
  ) -> FindBy<avatars::table, avatars::image_key, String> {
    avatars::table.filter(avatars::image_key.eq(image_key.to_string()))
  }

  pub fn filter_by_ids(ids: Vec<Uuid>) -> Filter<avatars::table, EqAny<avatars::id, Vec<Uuid>>> {
    avatars::table.filter(avatars::id.eq_any(ids))
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

  pub fn find_by_id(id: &Uuid) -> Filter<avatars::table, Eq<avatars::id, Uuid>> {
    avatars::table.filter(avatars::id.eq(*id))
  }
}
