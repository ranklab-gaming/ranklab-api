use crate::config::Config;
use crate::emails::{Email, Recipient};
use crate::guards::DbConn;
use crate::models::{Comment, Recording, User};
use crate::schema::{comments, users};
use clokwerk::{Scheduler, TimeUnits};
use diesel::dsl::now;
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
  let comments_query = Comment::filter_unnotified();

  let comments = db_conn
    .run(move |conn| comments_query.load::<Comment>(conn))
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

    if user.emails_enabled {
      let comments_added = Email::new(
        config,
        "notification".to_owned(),
        json!({
          "subject": "You've received comments on your VOD!",
          "title": format!("You've received {} on the VOD \"{}\"", pluralize("comments", comments.len().try_into()?, true), title),
          "body": format!("You can follow the link below to view {}.", match comments.len() {
            1 => "it",
            _ => "them",
          }),
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

    db_conn
      .run(move |conn| {
        diesel::update(comments_query)
          .set(comments::notified_at.eq(now))
          .get_result::<Comment>(conn)
      })
      .await?;
  }

  Ok(())
}

#[allow(dead_code, unused_variables, unreachable_code)]
async fn process_digests(db_conn: &DbConn, config: &Config) -> Result<(), anyhow::Error> {
  return Ok(());

  let users_query = User::all();

  let users = db_conn
    .run(move |conn| users_query.load::<User>(conn))
    .await?;

  for user in users {
    if user.emails_enabled {
      let user_id = user.id;
      let digest_notified_at = user.digest_notified_at;

      let recordings = db_conn
        .run(move |conn| {
          Recording::filter_for_digest(&user_id, &digest_notified_at).load::<Recording>(conn)
        })
        .await?;

      if !recordings.is_empty() {
        let recordings_added = Email::new(
          config,
          "notification".to_owned(),
          json!({
            "subject": "There are new VODs to review!",
            "title": format!("There {} new {} waiting for feedback", match recordings.len() {
              1 => "is 1".to_owned(),
              _ => format!("are {}", recordings.len()),
            }, pluralize("VOD", recordings.len().try_into()?, false)),
            "body": format!("You can follow the link below to view {}.", match recordings.len() {
              1 => "it",
              _ => "them",
            }),
            "cta" : format!("View {}", pluralize("VOD", recordings.len().try_into()?, false)),
            "cta_url" : format!("{}/dashboard", config.web_host),
          }),
          vec![Recipient::new(
            user.email,
            json!({
              "name": user.name,
            }),
          )],
        );

        recordings_added
          .deliver()
          .await
          .map_err(|e| anyhow::anyhow!("Failed to send email: {}", e))?;
      }
    }
  }

  db_conn
    .run(move |conn| {
      diesel::update(users_query)
        .set(users::digest_notified_at.eq(now))
        .get_result::<User>(conn)
    })
    .await?;

  Ok(())
}

impl CronFairing {
  pub fn fairing() -> impl Fairing {
    Self
  }

  async fn init(&self, rocket: &Rocket<Orbit>) {
    let config_1 = rocket.state::<Config>().unwrap().clone();
    let config_2 = rocket.state::<Config>().unwrap().clone();
    let db_conn_1 = Arc::new(DbConn::get_one(rocket).await.unwrap());
    let db_conn_2 = Arc::new(DbConn::get_one(rocket).await.unwrap());

    tokio::spawn(async move {
      let mut scheduler = Scheduler::new();

      scheduler.every(30.minutes()).run(move || {
        let db_conn: Arc<DbConn> = Arc::clone(&db_conn_1);
        let config = config_1.clone();

        tokio::spawn(async move {
          if let Err(e) = process_comments(&db_conn, &config).await {
            error!("[cron] {:?}", e);
            sentry::capture_error(e.root_cause());
          }
        });
      });

      scheduler.every(1.day()).run(move || {
        let db_conn: Arc<DbConn> = Arc::clone(&db_conn_2);
        let config = config_2.clone();

        tokio::spawn(async move {
          if let Err(e) = process_digests(&db_conn, &config).await {
            error!("[cron] {:?}", e);
            sentry::capture_error(e.root_cause());
          }
        });
      });

      loop {
        scheduler.run_pending();
        tokio::time::sleep(chrono::Duration::seconds(1).to_std().unwrap()).await;
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
