use crate::config::Config;
use crate::guards::DbConn;
use crate::models::Comment;
use chrono::Duration;
use clokwerk::Interval::*;
use clokwerk::{Job, Scheduler, TimeUnits};
use diesel::prelude::*;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::{tokio, Orbit, Rocket};
use std::sync::Arc;

#[derive(Clone)]
pub struct CronFairing;

impl CronFairing {
  pub fn fairing() -> impl Fairing {
    Self
  }

  async fn init(&self, rocket: &Rocket<Orbit>) {
    let config = rocket.state::<Config>().unwrap().clone();
    let db_conn = Arc::new(DbConn::get_one(rocket).await.unwrap());

    tokio::spawn(async move {
      let mut scheduler = Scheduler::new();

      scheduler.every(30.minutes()).run(move || {
        let db_conn: Arc<DbConn> = Arc::clone(&db_conn);

        tokio::spawn(async move {
          let comments = db_conn
            .run(move |conn| Comment::filter_unnotified().load::<Comment>(conn))
            .await
            .unwrap();
        });
      });

      loop {
        scheduler.run_pending();
        tokio::time::sleep(Duration::seconds(1).to_std().unwrap()).await;
      }
    });
  }
}

#[rocket::async_trait]
impl Fairing for CronFairing {
  fn info(&self) -> Info {
    Info {
      name: "cron",
      kind: Kind::Liftoff,
    }
  }

  async fn on_liftoff(&self, rocket: &Rocket<Orbit>) {
    self.init(rocket).await;
  }
}
