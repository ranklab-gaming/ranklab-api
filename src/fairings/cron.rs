use crate::config::Config;
use crate::emails::{Email, Recipient};
use crate::games;
use crate::guards::DbConn;
use crate::models::{Comment, Recording, User};
use crate::schema::{comments, users};
use clokwerk::{Scheduler, TimeUnits};
use diesel::dsl::now;
use diesel::prelude::*;
use pluralizer::pluralize;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::{tokio, Orbit, Rocket};
use serde::Serialize;
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

#[derive(Serialize)]
struct GameWithRecordingCount {
  name: String,
  count: i64,
  url: String,
}

async fn process_digests(db_conn: &DbConn, config: &Config) -> Result<(), anyhow::Error> {
  let users: Vec<(User, String)> = db_conn
    .run(move |conn| User::filter_for_digest().load::<(User, String)>(conn))
    .await?;

  let mut users_with_game_ids: Vec<(User, Vec<String>)> = vec![];

  for user in users {
    let user_id = user.0.id;
    let game_id = user.1;

    let user_with_game_ids = users_with_game_ids
      .iter_mut()
      .find(|(user, _)| user.id == user_id);

    match user_with_game_ids {
      Some((_, game_ids)) => game_ids.push(game_id),
      None => users_with_game_ids.push((user.0, vec![game_id])),
    }
  }

  for (user, game_ids) in users_with_game_ids {
    let user_id = user.id;
    let digest_notified_at = user.digest_notified_at;
    let email = user.email.clone();
    let recordings_game_ids = game_ids.clone();

    let recordings = db_conn
      .run(move |conn| {
        Recording::filter_for_digest(&user_id, &digest_notified_at, recordings_game_ids)
          .load::<Recording>(conn)
      })
      .await?;

    if recordings.len() == 0 {
      continue;
    }

    let games_with_recording_count: Vec<GameWithRecordingCount> =
      game_ids.iter().fold(vec![], |mut acc, game_id| {
        let count: i64 = recordings
          .clone()
          .into_iter()
          .filter(|recording| recording.game_id == *game_id)
          .count()
          .try_into()
          .unwrap_or(0);

        if count == 0 {
          return acc;
        }

        if let Some(game) = games::find(game_id) {
          acc.push(GameWithRecordingCount {
            name: game.name.clone(),
            count,
            url: format!("{}/directory/{}", config.web_host, game_id),
          });
        }

        acc
      });

    let recordings_added = Email::new(
      config,
      "digest".to_owned(),
      json!({
        "title": "New VODs are available to review.",
        "subject": "There are new VODs to review!",
        "games": games_with_recording_count,
        "cta_url" : format!("{}/directory", config.web_host),
      }),
      vec![Recipient::new(
        email,
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

  db_conn
    .run(move |conn| {
      diesel::update(User::all())
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
