use crate::config::Config;
use crate::emails::{Email, Recipient};
use crate::games;
use crate::guards::DbConn;
use crate::models::{Comment, Digest, DigestChangeset, Following, Recording, User};
use crate::schema::{comments, digests};
use chrono::Duration;
use clokwerk::{Scheduler, TimeUnits};
use diesel::dsl::now;
use diesel::prelude::*;
use itertools::Itertools;
use pluralizer::pluralize;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::{tokio, Orbit, Rocket};
use serde::Serialize;
use serde_json::json;
use std::convert::TryInto;
use std::sync::Arc;
use tokio::time::sleep;

#[derive(Clone)]
pub struct CronFairing;

async fn process_comments(db_conn: &DbConn, config: &Config) -> Result<(), anyhow::Error> {
  let comments_query = Comment::filter_unnotified();

  let joined_comments = db_conn
    .run(move |conn| comments_query.load::<(Comment, User, Recording)>(conn))
    .await?;

  let comments = joined_comments
    .clone()
    .into_iter()
    .map(|(comment, _, _)| comment)
    .collect::<Vec<_>>();

  if joined_comments.is_empty() {
    return Ok(());
  }

  let users = joined_comments
    .clone()
    .into_iter()
    .map(|(_, user, _)| user)
    .unique_by(|user| user.id)
    .collect::<Vec<_>>();

  let mut recipients: Vec<Recipient> = vec![];

  for user in &users {
    let recordings = joined_comments
      .clone()
      .into_iter()
      .filter(|(_, _, recording)| recording.user_id == user.id)
      .map(|(_, _, recording)| recording)
      .unique_by(|recording| recording.id)
      .collect::<Vec<_>>();

    for recording in &recordings {
      let comments = comments
        .clone()
        .into_iter()
        .filter(|comment| comment.recording_id == recording.id)
        .collect::<Vec<_>>();

      if comments.is_empty() {
        continue;
      }

      let title = recording.title.clone();
      let recording_id = recording.id.to_string();

      let recipient = Recipient::new(
        user.email.clone(),
        json!({
          "name": user.name,
          "title": format!("You've received {} on the VOD \"{}\"", pluralize("comments", comments.len().try_into()?, true), title),
          "body": format!("You can follow the link below to view {}.", match comments.len() {
            1 => "it",
            _ => "them",
          }),
          "cta_url" : format!("{}/recordings/{}", config.web_host, recording_id),
        }),
      );

      recipients.push(recipient);
    }
  }

  let email = Email::new(
    config,
    "notification".to_owned(),
    json!({
      "subject": "You've received comments on your VOD!",
      "cta" : "View comments",
    }),
    recipients,
  );

  email
    .deliver()
    .await
    .map_err(|e| anyhow::anyhow!("Failed to send comments notification email: {}", e))?;

  let comment_ids = comments
    .clone()
    .into_iter()
    .map(|comment| comment.id)
    .collect::<Vec<_>>();

  db_conn
    .run(move |conn| {
      diesel::update(comments::table.filter(comments::id.eq_any(comment_ids)))
        .set(comments::notified_at.eq(now))
        .get_result::<Comment>(conn)
    })
    .await?;

  Ok(())
}

#[derive(Serialize)]
struct DigestEmailGame {
  name: String,
  count: isize,
  vods_label: String,
  url: String,
}

async fn process_digests(db_conn: &DbConn, config: &Config) -> Result<(), anyhow::Error> {
  let last_digest = db_conn
    .run(move |conn| Digest::last().first::<Digest>(conn).optional())
    .await?;

  let recordings = db_conn
    .run(move |conn| Recording::filter_for_digest(last_digest).load::<Recording>(conn))
    .await?;

  if recordings.is_empty() {
    db_conn
      .run(move |conn| {
        diesel::insert_into(digests::table)
          .values(DigestChangeset::default().metadata(json!({})))
          .get_result::<Digest>(conn)
      })
      .await?;

    return Ok(());
  }

  let users = db_conn
    .run(move |conn| User::filter_for_digest().load::<User>(conn))
    .await?;

  let followings_users = users.clone();

  let followings = db_conn
    .run(move |conn| Following::filter_for_digest(followings_users).load::<Following>(conn))
    .await?;

  let mut recipients: Vec<Recipient> = vec![];

  for user in &users {
    let email = user.email.clone();
    let name = user.name.clone();

    let followings = followings
      .clone()
      .into_iter()
      .filter(|following| following.user_id == user.id)
      .collect::<Vec<_>>();

    let games = followings
      .clone()
      .into_iter()
      .map(|following| {
        let game = games::find(&following.game_id).unwrap();

        let count = recordings
          .clone()
          .into_iter()
          .filter(|recording| {
            recording.game_id == game.id.to_string() && recording.user_id != user.id
          })
          .count() as isize;

        DigestEmailGame {
          name: game.name.clone(),
          vods_label: format!(
            "new {}",
            match count {
              1 => "VOD",
              _ => "VODs",
            }
          ),
          count,
          url: format!("{}/directory/{}", config.web_host, game.id.to_string()),
        }
      })
      .filter(|game| game.count > 0)
      .collect::<Vec<_>>();

    if games.is_empty() {
      continue;
    }

    let count = games.iter().map(|game| game.count).sum::<isize>();

    let vods_label = format!(
      "{} new {}",
      count,
      match count {
        1 => "VOD",
        _ => "VODs",
      }
    );

    let recipient = Recipient::new(
      email,
      json!({
        "name": name,
        "games": games,
        "title": format!("{} available to review.", match count {
          1 => "A new VOD",
          _ => "New VODs"
        }),
        "subject": format!("There {} {} available to review.", match count {
          1 => "is",
          _ => "are"
        }, vods_label),
        "vods_label": vods_label,
      }),
    );

    recipients.push(recipient);
  }

  let email = Email::new(
    &config,
    "digest".to_owned(),
    json!({
      "cta_url" : format!("{}/directory", config.web_host),
    }),
    recipients,
  );

  let metadata = serde_json::to_value(&email)?;

  email
    .deliver()
    .await
    .map_err(|e| anyhow::anyhow!("Failed to send digest email: {}", e))?;

  db_conn
    .run(move |conn| {
      diesel::insert_into(digests::table)
        .values(DigestChangeset::default().metadata(metadata))
        .get_result::<Digest>(conn)
    })
    .await?;

  Ok(())
}

impl CronFairing {
  pub fn fairing() -> impl Fairing {
    Self
  }

  async fn init(&self, rocket: &Rocket<Orbit>) {
    let config = Arc::new(rocket.state::<Config>().unwrap().clone());
    let db_conn = Arc::new(DbConn::get_one(rocket).await.unwrap());

    tokio::spawn(async move {
      let mut scheduler = Scheduler::new();

      scheduler.every(30.minutes()).run({
        let db_conn = Arc::clone(&db_conn);
        let config = Arc::clone(&config);

        move || {
          let db_conn = Arc::clone(&db_conn);
          let config = Arc::clone(&config);

          tokio::spawn(async move {
            if let Err(e) = process_comments(&db_conn, &config).await {
              error!("[cron] {:?}", e);
              sentry::capture_error(e.root_cause());
            }
          });
        }
      });

      scheduler.every(1.day()).run({
        let db_conn = Arc::clone(&db_conn);
        let config = Arc::clone(&config);

        move || {
          let db_conn = Arc::clone(&db_conn);
          let config = Arc::clone(&config);

          tokio::spawn(async move {
            if let Err(e) = process_digests(&db_conn, &config).await {
              error!("[cron] {:?}", e);
              sentry::capture_error(e.root_cause());
            }
          });
        }
      });

      loop {
        scheduler.run_pending();
        sleep(Duration::seconds(1).to_std().unwrap()).await;
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
