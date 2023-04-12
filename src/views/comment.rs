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
  pub video_timestamp: Option<i32>,
  pub drawing: String,
  pub preview: String,
  pub metadata: Option<serde_json::Value>,
}

impl From<Comment> for CommentView {
  fn from(comment: Comment) -> Self {
    let preview = html2text::from_read(comment.body.as_bytes(), 100);

    CommentView {
      id: comment.id,
      review_id: comment.review_id,
      coach_id: comment.coach_id,
      body: comment.body,
      video_timestamp: comment.video_timestamp,
      drawing: comment.drawing,
      preview,
      metadata: comment.metadata,
    }
  }
}
