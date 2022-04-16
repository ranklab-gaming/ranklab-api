use crate::schema::comments;
use derive_builder::Builder;
use uuid::Uuid;

#[derive(Builder, Queryable, Identifiable)]
#[builder(derive(AsChangeset), pattern = "owned", name = "CommentChangeset")]
#[builder_struct_attr(table_name = "comments")]
pub struct Comment {
  pub body: String,
  pub coach_id: Uuid,
  pub drawing: String,
  pub id: Uuid,
  pub review_id: Uuid,
  pub video_timestamp: i32,
}
