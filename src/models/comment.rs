use serde::Serialize;
use uuid::Uuid;
use rocket_okapi::{openapi, openapi_get_routes as routes, JsonSchema};

#[derive(Queryable, Serialize, JsonSchema)]
pub struct Comment {
    pub id: Uuid,
    pub review_id: Uuid,
    pub user_id: Uuid,
    pub body: String,
    pub video_timestamp: i32,
}
