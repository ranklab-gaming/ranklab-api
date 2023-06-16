use crate::data_types::MediaState;
use crate::schema::audios;
use derive_builder::Builder;
use diesel::dsl::{And, Eq, Filter, FindBy};
use diesel::helper_types::EqAny;
use diesel::prelude::*;
use schemars::JsonSchema;
use serde::Serialize;
use uuid::Uuid;

#[derive(Builder, Queryable, Identifiable, Clone, Serialize, JsonSchema)]
#[builder(
  derive(AsChangeset, Insertable),
  pattern = "owned",
  name = "AudioChangeset"
)]
#[builder_struct_attr(diesel(table_name = audios))]
pub struct Audio {
  pub created_at: chrono::NaiveDateTime,
  pub id: Uuid,
  pub updated_at: chrono::NaiveDateTime,
  pub audio_key: String,
  pub processed_audio_key: Option<String>,
  pub state: MediaState,
  pub transcript: Option<String>,
  pub user_id: Uuid,
}

impl Audio {
  pub fn find_by_audio_key(audio_key: &str) -> FindBy<audios::table, audios::audio_key, String> {
    audios::table.filter(audios::audio_key.eq(audio_key.to_string()))
  }

  pub fn find_for_user(
    user_id: &Uuid,
    id: &Uuid,
  ) -> Filter<audios::table, And<Eq<audios::id, Uuid>, Eq<audios::user_id, Uuid>>> {
    audios::table.filter(audios::id.eq(*id).and(audios::user_id.eq(*user_id)))
  }

  pub fn filter_processed_by_ids(
    ids: Vec<Uuid>,
  ) -> Filter<audios::table, And<EqAny<audios::id, Vec<Uuid>>, Eq<audios::state, MediaState>>> {
    audios::table.filter(
      audios::id
        .eq_any(ids)
        .and(audios::state.eq(MediaState::Processed)),
    )
  }
}
