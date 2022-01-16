use diesel_as_jsonb::AsJsonb;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(AsJsonb, Debug, Deserialize, Serialize, JsonSchema, Clone)]
pub struct UserGame {
  pub game_id: String,
  pub skill_level: u8,
}
