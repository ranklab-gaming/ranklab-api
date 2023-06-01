use crate::data_types::MediaState;
use crate::schema::audios;
use derive_builder::Builder;
use diesel::dsl::FindBy;
use diesel::dsl::{And, Eq, Filter};
use diesel::helper_types::NotEq;
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
  pub review_id: Uuid,
  pub updated_at: chrono::NaiveDateTime,
  pub audio_key: String,
  pub processed_audio_key: Option<String>,
  pub state: MediaState,
  pub transcript: Option<String>,
}

impl Audio {
  pub fn find_by_audio_key<T: ToString>(
    audio_key: &T,
  ) -> FindBy<audios::table, audios::audio_key, String> {
    audios::table.filter(audios::audio_key.eq(audio_key.to_string()))
  }

  pub fn find_by_id(id: &Uuid) -> FindBy<audios::table, audios::id, Uuid> {
    audios::table.filter(audios::id.eq(*id))
  }

  #[allow(clippy::type_complexity)]
  pub fn find_for_review_id(
    id: &Uuid,
    review_id: &Uuid,
  ) -> Filter<
    audios::table,
    And<And<Eq<audios::id, Uuid>, Eq<audios::review_id, Uuid>>, NotEq<audios::state, MediaState>>,
  > {
    audios::table.filter(
      audios::id
        .eq(*id)
        .and(audios::review_id.eq(*review_id))
        .and(audios::state.ne(MediaState::Created)),
    )
  }

  #[allow(clippy::type_complexity)]
  pub fn filter_by_review_id(
    review_id: &Uuid,
  ) -> Filter<audios::table, And<Eq<audios::review_id, Uuid>, NotEq<audios::state, MediaState>>> {
    audios::table.filter(
      audios::review_id
        .eq(*review_id)
        .and(audios::state.ne(MediaState::Created)),
    )
  }
}
