use crate::schema::{comments, recordings, users};
use derive_builder::Builder;
use diesel::dsl::{And, Eq, Filter};
use diesel::helper_types::{InnerJoin, IsNull, On, Select};
use diesel::prelude::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, JsonSchema, Clone)]
#[serde(rename_all = "snake_case")]
pub enum CommentMetadataValue {
  Video { timestamp: i64, drawing: String },
}

#[derive(Serialize, Deserialize, JsonSchema, Clone)]
#[serde(untagged)]
pub enum CommentMetadata {
  Some(CommentMetadataValue),
  None {},
}

#[derive(Builder, Queryable, Identifiable, Clone)]
#[builder(
  derive(AsChangeset, Insertable),
  pattern = "owned",
  name = "CommentChangeset"
)]
#[builder_struct_attr(diesel(table_name = comments))]
pub struct Comment {
  pub body: String,
  pub created_at: chrono::NaiveDateTime,
  pub id: Uuid,
  pub updated_at: chrono::NaiveDateTime,
  pub metadata: serde_json::Value,
  pub user_id: Uuid,
  pub recording_id: Uuid,
  pub notified_at: Option<chrono::NaiveDateTime>,
}

#[allow(clippy::type_complexity)]
impl Comment {
  pub fn find_for_user(
    user_id: &Uuid,
    id: &Uuid,
  ) -> Filter<comments::table, And<Eq<comments::id, Uuid>, Eq<comments::user_id, Uuid>>> {
    comments::table.filter(comments::id.eq(*id).and(comments::user_id.eq(*user_id)))
  }

  pub fn filter_by_recording_id(
    recording_id: &Uuid,
  ) -> Filter<comments::table, Eq<comments::recording_id, Uuid>> {
    comments::table.filter(comments::recording_id.eq(*recording_id))
  }

  pub fn filter_unnotified() -> Select<
    Filter<
      InnerJoin<
        InnerJoin<
          comments::table,
          On<recordings::table, Eq<comments::recording_id, recordings::id>>,
        >,
        On<users::table, Eq<recordings::user_id, users::id>>,
      >,
      And<Eq<users::emails_enabled, bool>, IsNull<comments::notified_at>>,
    >,
    (
      <comments::table as diesel::Table>::AllColumns,
      <users::table as diesel::Table>::AllColumns,
      <recordings::table as diesel::Table>::AllColumns,
    ),
  > {
    comments::table
      .inner_join(recordings::table.on(comments::recording_id.eq(recordings::id)))
      .inner_join(users::table.on(recordings::user_id.eq(users::id)))
      .filter(
        users::emails_enabled
          .eq(true)
          .and(comments::notified_at.is_null()),
      )
      .select((
        comments::all_columns,
        users::all_columns,
        recordings::all_columns,
      ))
  }
}
