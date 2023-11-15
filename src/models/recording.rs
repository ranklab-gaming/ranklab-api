use crate::data_types::MediaState;
use crate::schema::{comments, recordings};
use chrono::{Duration, NaiveDateTime, Utc};
use derive_builder::Builder;
use diesel::dsl::{And, Eq, Filter, FindBy};
use diesel::expression::SqlLiteral;
use diesel::helper_types::{EqAny, GroupBy, Gt, LeftJoin, NotEq, On, Order, Select};
use diesel::prelude::*;
use diesel::sql_types::Bool;
use schemars::JsonSchema;
use serde::Serialize;
use uuid::Uuid;

use super::Digest;

#[derive(Builder, Queryable, Identifiable, Clone, Serialize, JsonSchema)]
#[builder(
  derive(AsChangeset, Insertable),
  pattern = "owned",
  name = "RecordingChangeset"
)]
#[builder_struct_attr(diesel(table_name = recordings))]
pub struct Recording {
  pub created_at: NaiveDateTime,
  pub game_id: String,
  pub id: Uuid,
  pub user_id: Uuid,
  pub skill_level: i16,
  pub title: String,
  pub updated_at: NaiveDateTime,
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
  pub fn find_processed_for_user(
    user_id: &Uuid,
    id: &Uuid,
  ) -> Filter<
    recordings::table,
    And<
      And<Eq<recordings::id, Uuid>, Eq<recordings::user_id, Uuid>>,
      Eq<recordings::state, MediaState>,
    >,
  > {
    recordings::table.filter(
      recordings::id
        .eq(*id)
        .and(recordings::user_id.eq(*user_id))
        .and(recordings::state.eq(MediaState::Processed)),
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
    last_digest: Option<Digest>,
  ) -> Filter<
    recordings::table,
    And<Gt<recordings::updated_at, NaiveDateTime>, Eq<recordings::state, MediaState>>,
  > {
    let last_digest_at = last_digest
      .map(|digest| digest.created_at)
      .unwrap_or_else(|| {
        Utc::now()
          .naive_utc()
          .checked_sub_signed(Duration::days(1))
          .unwrap()
      });

    recordings::table.filter(
      recordings::updated_at
        .gt(last_digest_at)
        .and(recordings::state.eq(MediaState::Processed)),
    )
  }

  pub fn filter_by_ids(
    ids: Vec<Uuid>,
  ) -> Filter<recordings::table, EqAny<recordings::id, Vec<Uuid>>> {
    recordings::table.filter(recordings::id.eq_any(ids))
  }
}
