#[macro_use]
extern crate rocket;

use ranklab_api::config::Config;
use ranklab_api::db::{run_migrations, DbConn};
use ranklab_api::routes;
use rocket::fairing::AdHoc;
use rocket::figment::providers::{Env, Format, Toml};
use rocket::http::Accept;
use rocket::{Build, Rocket};

#[launch]
fn rocket() -> Rocket<Build> {
    let mut figment = rocket::Config::figment()
        .merge(Toml::file("Ranklab.toml").nested())
        .merge(Env::prefixed("RANKLAB_").global());

    if let Some(database_url) = Env::var("DATABASE_URL") {
        figment = figment.merge(("databases.default.url", database_url));
    }

    rocket::custom(figment)
        .attach(DbConn::fairing())
        .attach(AdHoc::on_ignite("Run Migrations", run_migrations))
        .attach(AdHoc::on_request("Accept JSON", |req, _| {
            Box::pin(async move { req.replace_header(Accept::JSON) })
        }))
        .attach(AdHoc::config::<Config>())
        .mount("/", routes::index())
        .mount("/users", routes::users())
        .mount("/recordings", routes::recordings())
        .mount("/reviews", routes::reviews())
        .mount("/coaches", routes::coaches())
        .mount("/comments", routes::comments())
}
