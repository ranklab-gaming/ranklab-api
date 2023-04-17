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
  pub preview: String,
  pub metadata: serde_json::Value,
}

impl From<Comment> for CommentView {
  fn from(comment: Comment) -> Self {
    let preview = html2text::from_read(comment.body.as_bytes(), 100);

    CommentView {
      id: comment.id,
      review_id: comment.review_id,
      coach_id: comment.coach_id,
      body: comment.body,
      preview,
      metadata: comment.metadata,
    }
  }
}
