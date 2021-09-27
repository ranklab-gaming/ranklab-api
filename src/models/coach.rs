use serde::Serialize;
use uuid::Uuid;
use rocket_okapi::{openapi, openapi_get_routes as routes, JsonSchema};

#[derive(Queryable, Serialize, JsonSchema)]
pub struct Coach {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub email: String,
    pub bio: String,
    pub game: String,
}
