use serde::Serialize;
use uuid::Uuid;

#[derive(Queryable, Serialize)]
pub struct Comment {
    pub id: Uuid,
    pub review_id: Uuid,
    pub user_id: Uuid,
    pub body: String,
    pub video_timestamp: i32,
}
