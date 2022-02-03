use uuid::Uuid;

#[derive(Queryable)]
pub struct Comment {
  pub id: Uuid,
  pub review_id: Uuid,
  pub coach_id: Uuid,
  pub body: String,
  pub video_timestamp: i32,
  pub drawing: String,
}
