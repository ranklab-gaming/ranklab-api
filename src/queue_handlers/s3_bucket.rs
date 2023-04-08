use crate::config::Config;
use crate::data_types::RecordingState;
use crate::fairings::sqs::{QueueHandler, QueueHandlerError};
use crate::guards::DbConn;
use crate::models::{Recording, RecordingChangeset};
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
      let recording = Recording::find_by_video_key(&video_key);

      if folder == "originals" {
        self
          .db_conn
          .run::<_, diesel::result::QueryResult<_>>(move |conn| {
            diesel::update(recording)
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
            diesel::update(recording)
              .set(
                RecordingChangeset::default()
                  .state(RecordingState::Processed)
                  .processed_video_key(Some(record.s3.object.key)),
              )
              .execute(conn)
          })
          .await
          .map_err(QueueHandlerError::from)?;

        continue;
      }

      if file.ends_with(".jpg") {
        self
          .db_conn
          .run::<_, diesel::result::QueryResult<_>>(move |conn| {
            diesel::update(recording)
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
