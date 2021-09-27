use serde::Serialize;
use uuid::Uuid;
use rocket_okapi::{openapi, openapi_get_routes as routes, JsonSchema};

#[derive(Queryable, Serialize, JsonSchema)]
pub struct Review {
    pub id: Uuid,
    pub user_id: Uuid,
    pub coach_id: Option<Uuid>,
    pub title: String,
    pub video_url: String,
    pub game: String,
}
