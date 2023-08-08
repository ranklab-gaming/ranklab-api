use crate::config::Config;
use crate::emails::{Email, Recipient};
use crate::guards::DbConn;
use crate::models::{Comment, Recording, User};
use chrono::Duration;
use clokwerk::{Scheduler, TimeUnits};
use diesel::prelude::*;
use pluralizer::pluralize;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::{tokio, Orbit, Rocket};
use serde_json::json;
use std::collections::HashMap;
use std::convert::TryInto;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct CronFairing;

async fn process_comments(db_conn: &DbConn, config: &Config) -> Result<(), anyhow::Error> {
  let comments = db_conn
    .run(move |conn| Comment::filter_unnotified().load::<Comment>(conn))
    .await?;

  let mut comments_by_recording_id: HashMap<Uuid, Vec<Comment>> = HashMap::new();
  for comment in comments {
    let recording_id = comment.recording_id;
    let comments = comments_by_recording_id
      .entry(recording_id)
      .or_insert(vec![]);
    comments.push(comment);
  }

  for (recording_id, comments) in comments_by_recording_id {
    let recording = db_conn
      .run(move |conn| Recording::find_by_id(&recording_id).get_result::<Recording>(conn))
      .await?;

    let title = recording.title;
    let user_id = recording.user_id;

    let user = db_conn
      .run(move |conn| User::find_by_id(&user_id).get_result::<User>(conn))
      .await?;

    let comments_added = Email::new(
      config,
      "notification".to_owned(),
      json!({
        "subject": "You've received comments on your VOD!",
        "title": format!("You've received {} on the VOD \"{}\"", pluralize("comments", comments.len().try_into()?, true), title),
        "body": "You can follow the link below to view them.",
        "cta" : "View comments",
        "cta_url" : format!("{}/recordings/{}", config.web_host, recording_id),
      }),
      vec![Recipient::new(
        user.email,
        json!({
          "name": user.name,
        }),
      )],
    );

    comments_added
      .deliver()
      .await
      .map_err(|e| anyhow::anyhow!("Failed to send email: {}", e))?;
  }

  Ok(())
}

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
        let config = config.clone();

        tokio::spawn(async move {
          if let Err(e) = process_comments(&db_conn, &config).await {
            error!("[cron] {:?}", e);
            sentry::capture_error(e.root_cause());
          }
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
