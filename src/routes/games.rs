use crate::db::DbConn;
use crate::guards::Auth;
use crate::models::{Game, User};
use diesel::prelude::*;
use rocket::serde::json::Json;
use rocket_okapi::openapi;

#[openapi(tag = "Ranklab")]
#[get("/games")]
pub async fn list(_auth: Auth<User>, db_conn: DbConn) -> Json<Vec<Game>> {
    let games = db_conn
        .run(move |conn| {
            use crate::schema::games::dsl::*;
            games.load(conn).unwrap()
        })
        .await;

    Json(games)
}
