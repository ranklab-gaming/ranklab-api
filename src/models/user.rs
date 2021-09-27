use serde::Serialize;
use uuid::Uuid;
use rocket_okapi::{openapi, openapi_get_routes as routes, JsonSchema};

#[derive(Queryable, Serialize, JsonSchema)]
pub struct User {
    pub id: Uuid,
    pub auth0_id: String,
}
