use rocket::{Build, Rocket};
use rocket_sync_db_pools::database;
use rocket_okapi::impl_from_param;

#[database("default")]
pub struct DbConn(diesel::PgConnection);

impl_from_param!(DbConn);

pub async fn run_migrations(rocket: Rocket<Build>) -> Rocket<Build> {
    embed_migrations!();

    let db_conn = DbConn::get_one(&rocket)
        .await
        .expect("Failed to get db connection");

    db_conn
        .run(|c| embedded_migrations::run(c))
        .await
        .expect("Failed to run migrations");

    rocket
}
