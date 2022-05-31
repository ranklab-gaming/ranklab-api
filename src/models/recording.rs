use crate::schema::recordings;
use derive_builder::Builder;
use diesel::dsl::{And, Eq, Filter, FindBy};
use diesel::prelude::*;
use uuid::Uuid;

#[derive(Builder, Queryable, Identifiable, Clone)]
#[builder(
  derive(AsChangeset, Insertable),
  pattern = "owned",
  name = "RecordingChangeset"
)]
#[builder_struct_attr(table_name = "recordings")]
pub struct Recording {
  pub id: Uuid,
  pub mime_type: String,
  pub player_id: Uuid,
  pub upload_url: String,
  pub uploaded: bool,
  pub video_key: String,
  pub updated_at: chrono::NaiveDateTime,
  pub created_at: chrono::NaiveDateTime,
}

impl Recording {
  pub fn find_by_video_key<T: ToString>(
    video_key: &T,
  ) -> FindBy<recordings::table, recordings::video_key, String> {
    recordings::table.filter(recordings::video_key.eq(video_key.to_string()))
  }

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

  pub fn filter_for_player(
    player_id: &Uuid,
  ) -> Filter<recordings::table, Eq<recordings::player_id, Uuid>> {
    recordings::table.filter(recordings::player_id.eq(*player_id))
  }
}
