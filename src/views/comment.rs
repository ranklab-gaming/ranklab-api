use super::UserView;
use crate::models::{Comment, CommentMetadata, User};
use schemars::JsonSchema;
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize, JsonSchema)]
#[serde(rename = "Comment")]
pub struct CommentView {
  pub id: Uuid,
  pub recording_id: Uuid,
  pub user_id: Uuid,
  pub body: String,
  pub preview: String,
  pub metadata: CommentMetadata,
  pub user: Option<UserView>,
  pub created_at: chrono::NaiveDateTime,
}

impl From<Comment> for CommentView {
  fn from(comment: Comment) -> Self {
    CommentView::new(comment, None)
  }
}

impl CommentView {
  pub fn new(comment: Comment, user: Option<User>) -> Self {
    let preview = html2text::from_read(comment.body.as_bytes(), 100);

    CommentView {
      id: comment.id,
      recording_id: comment.recording_id,
      user_id: comment.user_id,
      body: comment.body,
      preview,
      metadata: serde_json::from_value(comment.metadata).unwrap(),
      user: user.map(UserView::from),
      created_at: comment.created_at,
    }
  }
}
