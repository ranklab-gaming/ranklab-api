use rocket::serde::Serialize;
use uuid::Uuid;

#[derive(Queryable, Serialize)]
pub struct Coach {
    pub id: Uuid,
    pub user_id: Uuid,
}
