use crate::data_types::MediaState;
use crate::schema::recordings;
use derive_builder::Builder;
use diesel::dsl::{And, Eq, Filter, FindBy};
use diesel::expression::SqlLiteral;
use diesel::helper_types::{NotEq, Order};
use diesel::prelude::*;
use diesel::sql_types::Bool;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Serialize, Deserialize, JsonSchema, Clone, Validate)]
pub struct RecordingOverwatchMetadata {
  #[validate(length(min = 6, max = 6))]
  pub replay_code: String,
  #[validate(range(min = 0, max = 9))]
  pub player_position: u8,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone)]
#[serde(rename_all = "snake_case")]
pub enum RecordingMetadataValue {
  Overwatch(RecordingOverwatchMetadata),
}

#[derive(Serialize, Deserialize, JsonSchema, Clone)]
#[serde(untagged)]
pub enum RecordingMetadata {
  Some(RecordingMetadataValue),
  None {},
}

impl RecordingMetadata {
  pub fn is_overwatch(&self) -> bool {
    match self {
      RecordingMetadata::Some(RecordingMetadataValue::Overwatch(_)) => true,
      _ => false,
    }
  }
}

impl Validate for RecordingMetadata {
  fn validate(&self) -> Result<(), validator::ValidationErrors> {
    match self {
      RecordingMetadata::Some(metadata) => match metadata {
        RecordingMetadataValue::Overwatch(metadata) => metadata.validate(),
      },
      RecordingMetadata::None {} => Ok(()),
    }
  }
}

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
  pub user_id: Uuid,
  pub skill_level: i16,
  pub title: String,
  pub updated_at: chrono::NaiveDateTime,
  pub video_key: Option<String>,
  pub thumbnail_key: Option<String>,
  pub processed_video_key: Option<String>,
  pub metadata: Option<serde_json::Value>,
  pub state: MediaState,
  pub notes: String,
}

impl Recording {
  pub fn all() -> Filter<recordings::table, SqlLiteral<Bool>> {
    recordings::table.filter(diesel::dsl::sql::<Bool>("true"))
  }

  pub fn find_by_video_key(
    video_key: &str,
  ) -> FindBy<recordings::table, recordings::video_key, String> {
    recordings::table.filter(recordings::video_key.eq(video_key.to_string()))
  }

  pub fn find_by_id(id: &Uuid) -> FindBy<recordings::table, recordings::id, Uuid> {
    recordings::table.filter(recordings::id.eq(*id))
  }

  #[allow(clippy::type_complexity)]
  pub fn find_for_user(
    user_id: &Uuid,
    id: &Uuid,
  ) -> Filter<recordings::table, And<Eq<recordings::id, Uuid>, Eq<recordings::user_id, Uuid>>> {
    recordings::table.filter(recordings::id.eq(*id).and(recordings::user_id.eq(*user_id)))
  }

  #[allow(clippy::type_complexity)]
  pub fn find_for_game(
    game_id: &str,
    id: &Uuid,
  ) -> Filter<recordings::table, And<Eq<recordings::id, Uuid>, Eq<recordings::game_id, String>>> {
    recordings::table.filter(
      recordings::id
        .eq(*id)
        .and(recordings::game_id.eq(game_id.to_string())),
    )
  }

  #[allow(clippy::type_complexity)]
  pub fn filter_for_user(
    user_id: &Uuid,
  ) -> Order<
    Filter<
      recordings::table,
      And<Eq<recordings::user_id, Uuid>, NotEq<recordings::state, MediaState>>,
    >,
    SqlLiteral<Bool>,
  > {
    recordings::table
      .filter(
        recordings::user_id
          .eq(*user_id)
          .and(recordings::state.ne(MediaState::Created)),
      )
      .order(diesel::dsl::sql::<Bool>("created_at desc"))
  }

  pub fn filter_by_game_id(
    game_id: &str,
  ) -> Order<
    Filter<
      recordings::table,
      And<
        Eq<crate::schema::recordings::game_id, String>,
        Eq<crate::schema::recordings::state, MediaState>,
      >,
    >,
    SqlLiteral<Bool>,
  > {
    recordings::table
      .filter(
        crate::schema::recordings::game_id
          .eq(game_id.to_string())
          .and(crate::schema::recordings::state.eq(MediaState::Processed)),
      )
      .order(diesel::dsl::sql::<Bool>("created_at desc"))
  }
}
