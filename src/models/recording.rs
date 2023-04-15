use crate::data_types::RecordingState;
use crate::schema::recordings;
use derive_builder::Builder;
use diesel::dsl::{And, Eq, EqAny, Filter, FindBy};
use diesel::helper_types::NotEq;
use diesel::prelude::*;
use schemars::JsonSchema;
use serde::Serialize;
use uuid::Uuid;

#[derive(Builder, Queryable, Identifiable, Clone, Serialize, JsonSchema)]
#[builder(
  derive(AsChangeset, Insertable),
  pattern = "owned",
  name = "RecordingChangeset"
)]
#[builder_struct_attr(diesel(table_name = recordings))]
pub struct Recording {
  pub created_at: chrono::NaiveDateTime,
  pub game_id: String,
  pub id: Uuid,
  pub player_id: Uuid,
  pub skill_level: i16,
  pub title: String,
  pub updated_at: chrono::NaiveDateTime,
  pub video_key: Option<String>,
  pub state: RecordingState,
  pub thumbnail_key: Option<String>,
  pub processed_video_key: Option<String>,
  pub metadata: Option<serde_json::Value>,
}

impl Recording {
  pub fn find_by_video_key<T: ToString>(
    video_key: &T,
  ) -> FindBy<recordings::table, recordings::video_key, String> {
    recordings::table.filter(recordings::video_key.eq(video_key.to_string()))
  }

  #[allow(clippy::type_complexity)]
  pub fn find_for_player(
    id: &Uuid,
    player_id: &Uuid,
  ) -> Filter<recordings::table, And<Eq<recordings::id, Uuid>, Eq<recordings::player_id, Uuid>>> {
    recordings::table.filter(
      recordings::id
        .eq(*id)
        .and(recordings::player_id.eq(*player_id)),
    )
  }

  #[allow(clippy::type_complexity)]
  pub fn find_for_player_by_game_id(
    id: &Uuid,
    player_id: &Uuid,
    game_id: &str,
  ) -> Filter<
    recordings::table,
    And<
      And<Eq<recordings::id, Uuid>, Eq<recordings::game_id, String>>,
      Eq<recordings::player_id, Uuid>,
    >,
  > {
    recordings::table.filter(
      recordings::id
        .eq(*id)
        .and(recordings::game_id.eq(game_id.to_string()))
        .and(recordings::player_id.eq(*player_id)),
    )
  }

  #[allow(clippy::type_complexity)]
  pub fn filter_for_player(
    player_id: &Uuid,
  ) -> Filter<
    recordings::table,
    And<Eq<recordings::player_id, Uuid>, NotEq<recordings::state, RecordingState>>,
  > {
    recordings::table.filter(
      recordings::player_id
        .eq(*player_id)
        .and(recordings::state.ne(RecordingState::Created)),
    )
  }

  pub fn filter_by_ids(
    ids: Vec<Uuid>,
  ) -> Filter<recordings::table, EqAny<recordings::id, Vec<Uuid>>> {
    recordings::table.filter(recordings::id.eq_any(ids))
  }

  pub fn find_by_id(id: &Uuid) -> FindBy<recordings::table, recordings::id, Uuid> {
    recordings::table.filter(recordings::id.eq(*id))
  }
}
