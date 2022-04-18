use crate::schema::comments;
use derive_builder::Builder;
use diesel::dsl::{And, Eq, Filter};
use diesel::prelude::*;
use uuid::Uuid;

#[derive(Builder, Queryable, Identifiable)]
#[builder(
  derive(AsChangeset, Insertable),
  pattern = "owned",
  name = "CommentChangeset"
)]
#[builder_struct_attr(table_name = "comments")]
pub struct Comment {
  pub body: String,
  pub coach_id: Uuid,
  pub drawing: String,
  pub id: Uuid,
  pub review_id: Uuid,
  pub video_timestamp: i32,
}

impl Comment {
  pub fn find_for_coach(
    id: &Uuid,
    coach_id: &Uuid,
  ) -> Filter<comments::table, And<Eq<comments::id, Uuid>, Eq<comments::coach_id, Uuid>>> {
    comments::table.filter(comments::id.eq(*id).and(comments::coach_id.eq(*coach_id)))
  }

  pub fn filter_by_review_for_coach(
    review_id: &Uuid,
    coach_id: &Uuid,
  ) -> Filter<comments::table, And<Eq<comments::review_id, Uuid>, Eq<comments::coach_id, Uuid>>> {
    comments::table.filter(
      comments::review_id
        .eq(*review_id)
        .and(comments::coach_id.eq(*coach_id)),
    )
  }
}
