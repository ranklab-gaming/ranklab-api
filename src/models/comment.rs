use crate::schema::comments;
use uuid::Uuid;

#[derive(Queryable, Identifiable)]
pub struct Comment {
  pub body: String,
  pub coach_id: Uuid,
  pub drawing: String,
  pub id: Uuid,
  pub review_id: Uuid,
  pub video_timestamp: i32,
}
