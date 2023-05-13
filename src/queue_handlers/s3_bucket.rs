use crate::config::Config;
use crate::data_types::{AvatarState, RecordingState, ReviewState};
use crate::fairings::sqs::{QueueHandler, QueueHandlerError};
use crate::guards::DbConn;
use crate::models::{
  Avatar, AvatarChangeset, Coach, CoachChangeset, Recording, RecordingChangeset, Review,
};
use crate::{aws, emails};
use anyhow::anyhow;
use diesel::prelude::*;
use rusoto_core::HttpClient;
use rusoto_s3::{GetObjectRequest, S3Client, S3};
use rusoto_signature::Region;
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
  client: S3Client,
}

#[async_trait]
impl QueueHandler for S3BucketHandler {
  fn new(db_conn: DbConn, config: Config) -> Self {
    let mut builder = hyper::Client::builder();
    let aws_access_key_id = config.aws_access_key_id.clone();
    let aws_secret_key = config.aws_secret_key.clone();

    builder.pool_max_idle_per_host(0);

    let client = S3Client::new_with(
      HttpClient::from_builder(builder, hyper_tls::HttpsConnector::new()),
      aws::CredentialsProvider::new(aws_access_key_id, aws_secret_key),
      Region::EuWest2,
    );

    Self {
      db_conn,
      config,
      client,
    }
  }

  fn url(&self) -> String {
    self.config.s3_bucket_queue.clone()
  }

  async fn instance_id(
    &self,
    message: &rusoto_sqs::Message,
    _profile: &rocket::figment::Profile,
  ) -> Result<Option<String>, QueueHandlerError> {
    let message_body = self.parse_message(message)?;

    let key = message_body
      .records
      .first()
      .ok_or_else(|| anyhow!("No records found in sqs message"))?
      .s3
      .object
      .key
      .clone();

    let object = self
      .client
      .get_object(GetObjectRequest {
        bucket: self.config.s3_bucket.clone(),
        key: key.clone(),
        ..Default::default()
      })
      .await
      .map_err(anyhow::Error::from)?;

    let instance_id: Option<String> = object
      .metadata
      .and_then(|metadata| metadata.get("instance-id").cloned());

    Ok(instance_id)
  }

  async fn handle(
    &self,
    message: &rusoto_sqs::Message,
    profile: &rocket::figment::Profile,
  ) -> Result<(), QueueHandlerError> {
    let message_body = self.parse_message(message)?;

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
        return self
          .handle_recording_uploaded(&record, folder, file, profile)
          .await;
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
    profile: &rocket::figment::Profile,
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

      if profile == "test" {
        return Ok(());
      }

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

    if file.ends_with(".jpg") {
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

impl S3BucketHandler {
  fn parse_message(
    &self,
    message: &rusoto_sqs::Message,
  ) -> Result<SqsMessageBody, QueueHandlerError> {
    let body = message
      .body
      .clone()
      .ok_or_else(|| anyhow!("No body found in sqs message"))?;

    let message_body: SqsMessageBody = serde_json::from_str(&body).map_err(anyhow::Error::from)?;

    Ok(message_body)
  }
}
