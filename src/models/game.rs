use serde::{Deserialize, Serialize};
use rocket_okapi::{openapi, openapi_get_routes as routes, JsonSchema};

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Game {
    Overwatch,
    Chess,
}
