use crate::schema::recordings;
use derive_builder::Builder;
use diesel::dsl::FindBy;
use diesel::prelude::*;
use uuid::Uuid;

#[derive(Builder, Queryable, Identifiable)]
#[builder(derive(AsChangeset), pattern = "owned", name = "RecordingChangeset")]
#[builder_struct_attr(table_name = "recordings")]
pub struct Recording {
  pub id: Uuid,
  pub mime_type: String,
  pub player_id: Uuid,
  pub upload_url: String,
  pub uploaded: bool,
  pub video_key: String,
}

impl Recording {
  pub fn find_by_video_key<T: ToString>(
    video_key: T,
  ) -> FindBy<recordings::table, recordings::video_key, String> {
    recordings::table.filter(recordings::video_key.eq(video_key.to_string()))
  }
}
