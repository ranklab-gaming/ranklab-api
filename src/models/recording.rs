use crate::data_types::MediaState;
use crate::schema::{comments, recordings};
use derive_builder::Builder;
use diesel::dsl::{And, Eq, Filter, FindBy};
use diesel::expression::SqlLiteral;
use diesel::helper_types::{GroupBy, Gt, LeftJoin, NotEq, On, Order, Select};
use diesel::prelude::*;
use diesel::sql_types::Bool;
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
  pub user_id: Uuid,
  pub skill_level: i16,
  pub title: String,
  pub updated_at: chrono::NaiveDateTime,
  pub video_key: Option<String>,
  pub thumbnail_key: Option<String>,
  pub processed_video_key: Option<String>,
  pub state: MediaState,
  pub notes: String,
}

#[derive(Queryable, Clone, Serialize, JsonSchema)]
pub struct RecordingWithCommentCount {
  pub recording: Recording,
  pub comment_count: i64,
}

impl Recording {
  pub fn all() -> Select<
    GroupBy<
      LeftJoin<
        Order<Filter<recordings::table, Eq<recordings::state, MediaState>>, SqlLiteral<Bool>>,
        On<comments::table, Eq<comments::recording_id, recordings::id>>,
      >,
      recordings::id,
    >,
    (
      <recordings::table as diesel::Table>::AllColumns,
      diesel::dsl::count<diesel::dsl::Nullable<comments::id>>,
    ),
  > {
    recordings::table
      .filter(recordings::state.eq(MediaState::Processed))
      .order(diesel::dsl::sql::<Bool>("created_at desc"))
      .left_join(comments::table.on(comments::recording_id.eq(recordings::id)))
      .group_by(recordings::id)
      .select((
        recordings::all_columns,
        diesel::dsl::count(comments::id.nullable()),
      ))
  }

  pub fn filter_by_game_id(
    game_id: &str,
  ) -> Select<
    GroupBy<
      LeftJoin<
        Order<
          Filter<
            recordings::table,
            And<Eq<recordings::state, MediaState>, Eq<recordings::game_id, String>>,
          >,
          SqlLiteral<Bool>,
        >,
        On<comments::table, Eq<comments::recording_id, recordings::id>>,
      >,
      recordings::id,
    >,
    (
      <recordings::table as diesel::Table>::AllColumns,
      diesel::dsl::count<diesel::dsl::Nullable<comments::id>>,
    ),
  > {
    recordings::table
      .filter(
        recordings::state
          .eq(MediaState::Processed)
          .and(recordings::game_id.eq(game_id.to_string())),
      )
      .order(diesel::dsl::sql::<Bool>("created_at desc"))
      .left_join(comments::table.on(comments::recording_id.eq(recordings::id)))
      .group_by(recordings::id)
      .select((
        recordings::all_columns,
        diesel::dsl::count(comments::id.nullable()),
      ))
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
  ) -> Select<
    GroupBy<
      LeftJoin<
        Order<
          Filter<
            recordings::table,
            And<NotEq<recordings::state, MediaState>, Eq<recordings::user_id, Uuid>>,
          >,
          SqlLiteral<Bool>,
        >,
        On<comments::table, Eq<comments::recording_id, recordings::id>>,
      >,
      recordings::id,
    >,
    (
      <recordings::table as diesel::Table>::AllColumns,
      diesel::dsl::count<diesel::dsl::Nullable<comments::id>>,
    ),
  > {
    recordings::table
      .filter(
        recordings::state
          .ne(MediaState::Created)
          .and(recordings::user_id.eq(*user_id)),
      )
      .order(diesel::dsl::sql::<Bool>("created_at desc"))
      .left_join(comments::table.on(comments::recording_id.eq(recordings::id)))
      .group_by(recordings::id)
      .select((
        recordings::all_columns,
        diesel::dsl::count(comments::id.nullable()),
      ))
  }

  pub fn filter_for_digest(
    user_id: &Uuid,
    digest_notified_at: &chrono::NaiveDateTime,
  ) -> Filter<
    recordings::table,
    And<
      And<NotEq<recordings::user_id, Uuid>, Gt<recordings::created_at, chrono::NaiveDateTime>>,
      Eq<recordings::state, MediaState>,
    >,
  > {
    recordings::table.filter(
      recordings::user_id
        .ne(*user_id)
        .and(recordings::created_at.gt(*digest_notified_at))
        .and(recordings::state.eq(MediaState::Processed)),
    )
  }
}
