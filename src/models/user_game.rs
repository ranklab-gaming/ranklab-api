use diesel_as_jsonb::AsJsonb;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(AsJsonb, Debug, Deserialize, Serialize, JsonSchema, Clone)]
pub struct UserGame {
  game_id: String,
  skil_level: u8,
}
