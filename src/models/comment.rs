use crate::schema::comments;
use derive_builder::Builder;
use diesel::dsl::{And, Eq, Filter};
use diesel::helper_types::{Asc, Order};
use diesel::prelude::*;
use uuid::Uuid;

#[derive(Builder, Queryable, Identifiable, Clone)]
#[builder(
  derive(AsChangeset, Insertable),
  pattern = "owned",
  name = "CommentChangeset"
)]
#[builder_struct_attr(diesel(table_name = comments))]
pub struct Comment {
  pub body: String,
  pub coach_id: Uuid,
  pub created_at: chrono::NaiveDateTime,
  pub drawing: String,
  pub id: Uuid,
  pub review_id: Uuid,
  pub updated_at: chrono::NaiveDateTime,
  pub video_timestamp: Option<i32>,
  pub metadata: Option<serde_json::Value>,
}

#[allow(clippy::type_complexity)]
impl Comment {
  pub fn find_for_coach(
    id: &Uuid,
    coach_id: &Uuid,
  ) -> Filter<comments::table, And<Eq<comments::id, Uuid>, Eq<comments::coach_id, Uuid>>> {
    comments::table.filter(comments::id.eq(*id).and(comments::coach_id.eq(*coach_id)))
  }

  pub fn filter_by_review_id(
    review_id: &Uuid,
  ) -> Order<Filter<comments::table, Eq<comments::review_id, Uuid>>, Asc<comments::video_timestamp>>
  {
    comments::table
      .filter(comments::review_id.eq(*review_id))
      .order(comments::video_timestamp.asc())
  }

  pub fn filter_by_review_for_coach(
    review_id: &Uuid,
    coach_id: &Uuid,
  ) -> Order<
    Filter<comments::table, And<Eq<comments::review_id, Uuid>, Eq<comments::coach_id, Uuid>>>,
    Asc<comments::video_timestamp>,
  > {
    comments::table
      .filter(
        comments::review_id
          .eq(*review_id)
          .and(comments::coach_id.eq(*coach_id)),
      )
      .order(comments::video_timestamp.asc())
  }
}
