mod handlers;
use self::handlers::*;
use crate::aws;
use crate::config::Config;
use crate::fairings::sqs::{QueueHandler, QueueHandlerError};
use crate::guards::DbConn;
use anyhow::anyhow;
use rusoto_core::HttpClient;
use rusoto_s3::{HeadObjectRequest, S3Client, S3};
use rusoto_signature::Region;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct RecordS3Object {
  key: String,
}

#[derive(Deserialize)]
pub struct RecordS3 {
  object: RecordS3Object,
}

#[derive(Deserialize)]
pub struct Record {
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

#[derive(Deserialize)]
pub struct WhisperApiResponse {
  pub text: String,
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
      .head_object(HeadObjectRequest {
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
        return handle_recording_uploaded(&self, &record, folder, file, profile).await;
      }

      if file_type == "avatars" {
        handle_avatar_uploaded(&self, &record, folder, file).await?;
      }
    }

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
