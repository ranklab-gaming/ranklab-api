use crate::schema::digests;
use derive_builder::Builder;
use diesel::dsl::{Limit, Order};
use diesel::helper_types::Desc;
use diesel::prelude::*;
use uuid::Uuid;

#[derive(Builder, Queryable, Identifiable, Clone)]
#[builder(
  derive(AsChangeset, Insertable),
  pattern = "owned",
  name = "DigestChangeset"
)]
#[builder_struct_attr(diesel(table_name = digests))]
pub struct Digest {
  pub id: Uuid,
  pub created_at: chrono::NaiveDateTime,
  pub updated_at: chrono::NaiveDateTime,
  pub metadata: serde_json::Value,
}

#[allow(clippy::type_complexity)]
impl Digest {
  pub fn last() -> Limit<Order<digests::table, Desc<digests::created_at>>> {
    digests::table.order(digests::created_at.desc()).limit(1)
  }
}
