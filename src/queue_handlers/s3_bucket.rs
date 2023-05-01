use crate::config::Config;
use crate::data_types::{AvatarState, RecordingState, ReviewState};
use crate::emails;
use crate::fairings::sqs::{QueueHandler, QueueHandlerError};
use crate::guards::DbConn;
use crate::models::{
  Avatar, AvatarChangeset, Coach, CoachChangeset, Recording, RecordingChangeset, Review,
};
use anyhow::anyhow;
use diesel::prelude::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct RecordS3Object {
  key: String,
}

#[derive(Deserialize)]
struct RecordS3 {
  object: RecordS3Object,
}

#[derive(Deserialize)]
struct Record {
  s3: RecordS3,
}

#[derive(Deserialize)]
struct SqsMessageBody {
  #[serde(rename = "Records")]
  records: Vec<Record>,
}

pub struct S3BucketHandler {
  db_conn: DbConn,
  config: Config,
}

#[async_trait]
impl QueueHandler for S3BucketHandler {
  fn new(db_conn: DbConn, config: Config) -> Self {
    Self { db_conn, config }
  }

  fn url(&self) -> String {
    self.config.s3_bucket_queue.clone()
  }

  async fn handle(
    &self,
    message: &rusoto_sqs::Message,
    _profile: &rocket::figment::Profile,
  ) -> Result<(), QueueHandlerError> {
    let body = message
      .body
      .clone()
      .ok_or_else(|| anyhow!("No body found in sqs message"))?;

    let message_body: SqsMessageBody = serde_json::from_str(&body).map_err(anyhow::Error::from)?;

    for record in message_body.records {
      let mut parts = record
        .s3
        .object
        .key
        .split('/')
        .collect::<Vec<_>>()
        .into_iter();

      let file_type = parts
        .next()
        .ok_or_else(|| anyhow!("No file type found in s3 key"))?;

      let folder = parts
        .next()
        .ok_or_else(|| anyhow!("No folder found in s3 key"))?;

      let file = parts
        .next()
        .ok_or_else(|| anyhow!("No file found in s3 key"))?;

      if file_type == "recordings" {
        return self.handle_recording_uploaded(&record, folder, file).await;
      }

      if file_type == "avatars" {
        self.handle_avatar_uploaded(&record, folder, file).await?;
      }
    }

    Ok(())
  }
}

impl S3BucketHandler {
  async fn handle_recording_uploaded(
    &self,
    record: &Record,
    folder: &str,
    file: &str,
  ) -> Result<(), QueueHandlerError> {
    let video_key = format!(
      "recordings/originals/{}",
      file.split('_').collect::<Vec<_>>()[0]
    );

    let recording_query = Recording::find_by_video_key(&video_key);

    if folder == "originals" {
      self
        .db_conn
        .run::<_, diesel::result::QueryResult<_>>(move |conn| {
          diesel::update(recording_query)
            .set(RecordingChangeset::default().state(RecordingState::Uploaded))
            .execute(conn)
        })
        .await
        .map_err(QueueHandlerError::from)?;

      return Ok(());
    }

    if folder != "processed" {
      return Ok(());
    }

    if file.ends_with(".mp4") {
      let processed_video_key = Some(record.s3.object.key.clone());

      self
        .db_conn
        .run::<_, diesel::result::QueryResult<_>>(move |conn| {
          diesel::update(recording_query)
            .set(
              RecordingChangeset::default()
                .state(RecordingState::Processed)
                .processed_video_key(processed_video_key),
            )
            .execute(conn)
        })
        .await
        .map_err(QueueHandlerError::from)?;

      let recording = self
        .db_conn
        .run(move |conn| Recording::find_by_video_key(&video_key).first::<Recording>(conn))
        .await
        .map_err(QueueHandlerError::from)?;

      let reviews = self
        .db_conn
        .run(move |conn| Review::filter_by_recording_id(&recording.id).load::<Review>(conn))
        .await
        .map_err(QueueHandlerError::from)?;

      let coach_ids = reviews
        .iter()
        .map(|review| review.coach_id)
        .collect::<Vec<_>>();

      let coaches = self
        .db_conn
        .run(move |conn| Coach::filter_by_ids(coach_ids).load::<Coach>(conn))
        .await
        .map_err(QueueHandlerError::from)?;

      for review in reviews {
        let coach = coaches
          .iter()
          .find(|coach| coach.id == review.coach_id)
          .ok_or_else(|| anyhow!("No coach found for review"))?;

        if coach.emails_enabled && review.state == ReviewState::AwaitingReview {
          emails::notifications::coach_has_reviews(&self.config, coach)
            .deliver()
            .await
            .map_err(anyhow::Error::from)?;
        }
      }

      return Ok(());
    }

    if file.ends_with("1.jpg") {
      let thumbnail_key = Some(record.s3.object.key.clone());

      self
        .db_conn
        .run::<_, diesel::result::QueryResult<_>>(move |conn| {
          diesel::update(recording_query)
            .set(RecordingChangeset::default().thumbnail_key(thumbnail_key))
            .execute(conn)
        })
        .await
        .map_err(QueueHandlerError::from)?;
    }

    Ok(())
  }

  async fn handle_avatar_uploaded(
    &self,
    record: &Record,
    folder: &str,
    file: &str,
  ) -> Result<(), QueueHandlerError> {
    let image_key = format!(
      "avatars/originals/{}",
      file.split('.').collect::<Vec<_>>()[0]
    );

    let avatar: Avatar = self
      .db_conn
      .run(move |conn| Avatar::find_by_image_key(&image_key).first::<Avatar>(conn))
      .await?;

    if folder == "originals" {
      self
        .db_conn
        .run::<_, diesel::result::QueryResult<_>>(move |conn| {
          diesel::update(&avatar)
            .set(AvatarChangeset::default().state(AvatarState::Uploaded))
            .execute(conn)
        })
        .await
        .map_err(QueueHandlerError::from)?;

      return Ok(());
    }

    if folder != "processed" {
      return Ok(());
    }

    let processed_image_key = Some(record.s3.object.key.clone());

    let avatar: Avatar = self
      .db_conn
      .run::<_, diesel::result::QueryResult<_>>(move |conn| {
        diesel::update(&avatar)
          .set(
            AvatarChangeset::default()
              .state(AvatarState::Processed)
              .processed_image_key(processed_image_key),
          )
          .get_result(conn)
      })
      .await
      .map_err(QueueHandlerError::from)?;

    self
      .db_conn
      .run(move |conn| {
        diesel::update(Coach::find_by_id(&avatar.coach_id))
          .set(CoachChangeset::default().avatar_id(Some(avatar.id)))
          .execute(conn)
      })
      .await
      .map_err(QueueHandlerError::from)?;

    Ok(())
  }
}
