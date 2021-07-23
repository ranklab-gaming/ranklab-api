use serde::Serialize;
use uuid::Uuid;

#[derive(Queryable, Serialize)]
pub struct Coach {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub email: String,
    pub bio: String,
    pub game: String,
}
