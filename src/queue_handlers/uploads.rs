mod handlers;
use crate::aws;
use crate::config::Config;
use crate::fairings::sqs::{QueueHandler, QueueHandlerError};
use crate::guards::DbConn;
use anyhow::anyhow;
use rusoto_core::HttpClient;
use rusoto_s3::{HeadObjectRequest, S3Client, S3};
use rusoto_signature::Region;
use serde::Deserialize;

use self::handlers::{
  handle_avatar_processed, handle_avatar_uploaded, handle_recording_processed,
  handle_recording_uploaded,
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct S3EventRecordObject {
  key: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct S3EventRecordInfo {
  object: S3EventRecordObject,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct S3EventRecord {
  s3: S3EventRecordInfo,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct S3Event {
  records: Vec<S3EventRecord>,
}

pub struct UploadsHandler {
  db_conn: DbConn,
  config: Config,
  client: S3Client,
}

#[async_trait]
impl QueueHandler for UploadsHandler {
  fn name(&self) -> &'static str {
    "uploads"
  }

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
    self.config.uploads_queue_url.clone()
  }

  async fn instance_id(&self, message: String) -> Result<Option<String>, QueueHandlerError> {
    let message_body = self.message_body(message)?;

    let record = message_body
      .records
      .first()
      .ok_or_else(|| anyhow!("No records found in sqs message"))?;

    let original_key = self.original_key(record.s3.object.key.clone())?;

    let object = self
      .client
      .head_object(HeadObjectRequest {
        bucket: self.config.uploads_bucket.clone(),
        key: original_key.to_string(),
        ..Default::default()
      })
      .await
      .map_err(anyhow::Error::from)?;

    let instance_id: Option<String> = object
      .metadata
      .and_then(|metadata| metadata.get("instance-id").cloned());

    Ok(instance_id)
  }

  async fn handle(&self, message: String) -> Result<(), QueueHandlerError> {
    let message_body = self.message_body(message.clone())?;

    for record in message_body.records {
      let key = record.s3.object.key;
      let original_key = self.original_key(key.clone())?;

      if key.starts_with("avatars/originals/") {
        handle_avatar_uploaded(&self, key, message.clone()).await?;
      } else if key.starts_with("recordings/originals/") {
        handle_recording_uploaded(&self, key).await?;
      } else if key.starts_with("avatars/processed/") {
        handle_avatar_processed(&self, key, original_key).await?;
      } else if key.starts_with("recordings/processed/") {
        handle_recording_processed(&self, key, original_key).await?;
      }
    }

    Ok(())
  }
}

impl UploadsHandler {
  fn original_key(&self, key: String) -> Result<String, QueueHandlerError> {
    let original_key = key.replace("/processed", "/originals");
    let original_key = original_key.split('_').next().unwrap_or_default();
    let original_key = original_key.split('.').next().unwrap_or_default();

    Ok(original_key.to_string())
  }

  fn message_body(&self, message: String) -> Result<S3Event, QueueHandlerError> {
    let body: S3Event = serde_json::from_str(&message).map_err(anyhow::Error::from)?;
    Ok(body)
  }

  pub async fn delete_upload(&self, key: String) -> Result<(), QueueHandlerError> {
    self
      .client
      .delete_object(rusoto_s3::DeleteObjectRequest {
        bucket: self.config.uploads_bucket.clone(),
        key: key.clone(),
        ..Default::default()
      })
      .await
      .map_err(anyhow::Error::from)?;

    Ok(())
  }
}
