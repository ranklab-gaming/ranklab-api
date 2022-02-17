use crate::schema::recordings;
use uuid::Uuid;

#[derive(Queryable, Identifiable)]
pub struct Recording {
  pub id: Uuid,
  pub mime_type: String,
  pub player_id: Uuid,
  pub upload_url: String,
  pub uploaded: bool,
  pub video_key: String,
}
