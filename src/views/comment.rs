use crate::models::Comment;
use schemars::JsonSchema;
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize, JsonSchema)]
#[serde(rename = "Comment")]
pub struct CommentView {
  pub id: Uuid,
  pub review_id: Uuid,
  pub coach_id: Uuid,
  pub body: String,
  pub video_timestamp: i32,
  pub drawing: String,
}

impl From<Comment> for CommentView {
  fn from(comment: Comment) -> Self {
    CommentView {
      id: comment.id,
      review_id: comment.review_id,
      coach_id: comment.coach_id,
      body: comment.body,
      video_timestamp: comment.video_timestamp,
      drawing: comment.drawing,
    }
  }
}
