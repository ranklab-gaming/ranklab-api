use uuid::Uuid;

#[derive(Queryable)]
pub struct Recording {
  pub id: Uuid,
  pub player_id: Uuid,
  pub video_key: String,
  pub upload_url: String,
  pub uploaded: bool,
  pub mime_type: String,
}
