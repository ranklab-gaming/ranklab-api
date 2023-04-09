use crate::config::Config;
use crate::data_types::{RecordingState, ReviewState};
use crate::emails;
use crate::fairings::sqs::{QueueHandler, QueueHandlerError};
use crate::guards::DbConn;
use crate::models::{Coach, Recording, RecordingChangeset, Review};
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
      let parts = record.s3.object.key.split('/').collect::<Vec<_>>();

      let folder = *parts
        .first()
        .ok_or_else(|| anyhow!("No folder found in s3 key"))?;

      let file = *parts
        .get(1)
        .ok_or_else(|| anyhow!("No file found in s3 key"))?;

      let video_key = format!("originals/{}", file.split('_').collect::<Vec<_>>()[0]);
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

        continue;
      }

      if folder != "processed" {
        continue;
      }

      if file.ends_with(".mp4") {
        self
          .db_conn
          .run::<_, diesel::result::QueryResult<_>>(move |conn| {
            diesel::update(recording_query)
              .set(
                RecordingChangeset::default()
                  .state(RecordingState::Processed)
                  .processed_video_key(Some(record.s3.object.key)),
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

        continue;
      }

      if file.ends_with(".jpg") {
        self
          .db_conn
          .run::<_, diesel::result::QueryResult<_>>(move |conn| {
            diesel::update(recording_query)
              .set(RecordingChangeset::default().thumbnail_key(Some(record.s3.object.key)))
              .execute(conn)
          })
          .await
          .map_err(QueueHandlerError::from)?;

        continue;
      }
    }

    Ok(())
  }
}
