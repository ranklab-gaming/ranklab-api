use crate::data_types::AvatarState;
use crate::schema::avatars;
use derive_builder::Builder;
use diesel::dsl::{EqAny, Filter, FindBy};
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
  pub state: AvatarState,
  pub created_at: chrono::NaiveDateTime,
  pub updated_at: chrono::NaiveDateTime,
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

  pub fn find_by_id(id: &Uuid) -> FindBy<avatars::table, avatars::id, Uuid> {
    avatars::table.filter(avatars::id.eq(*id))
  }
}
