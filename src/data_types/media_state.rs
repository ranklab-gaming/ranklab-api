use diesel_derive_enum::DbEnum;
use rocket_okapi::JsonSchema;
use serde::Serialize;

#[derive(DbEnum, Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, JsonSchema)]
#[ExistingTypePath = "crate::schema::sql_types::MediaState"]
pub enum MediaState {
  Created,
  Uploaded,
  Processed,
}
