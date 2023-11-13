use crate::aws;
use crate::data_types::MediaState;
use crate::fairings::sqs::QueueHandlerError;
use crate::models::{Recording, RecordingChangeset};
use crate::queue_handlers::UploadsHandler;
use diesel::prelude::*;
use hyper_tls::HttpsConnector;
use rusoto_core::HttpClient;
use rusoto_rekognition::Rekognition;
use rusoto_signature::Region;

pub async fn handle_recording_uploaded(
  handler: &UploadsHandler,
  key: String,
) -> Result<(), QueueHandlerError> {
  let config = &handler.config;
  let video_key = key.clone();
  let builder = hyper::Client::builder();

  let rekognition = rusoto_rekognition::RekognitionClient::new_with(
    HttpClient::from_builder(builder, HttpsConnector::new()),
    aws::CredentialsProvider::new(
      config.aws_access_key_id.clone(),
      config.aws_secret_key.clone(),
    ),
    Region::EuWest2,
  );

  handler
    .db_conn
    .run::<_, diesel::result::QueryResult<_>>(move |conn| {
      diesel::update(Recording::find_by_video_key(&video_key))
        .set(RecordingChangeset::default().state(MediaState::Uploaded))
        .execute(conn)
    })
    .await
    .map_err(QueueHandlerError::from)?;

  let job = rekognition
    .start_content_moderation(rusoto_rekognition::StartContentModerationRequest {
      video: rusoto_rekognition::Video {
        s3_object: Some(rusoto_rekognition::S3Object {
          bucket: Some(config.uploads_bucket.clone()),
          name: Some(key.clone()),
          ..Default::default()
        }),
        ..Default::default()
      },
      notification_channel: Some(rusoto_rekognition::NotificationChannel {
        role_arn: config.rekognition_role_arn.clone(),
        sns_topic_arn: config.rekognition_topic_arn.clone(),
      }),
      ..Default::default()
    })
    .await
    .map_err(anyhow::Error::from)?;

  info!("Started content moderation job: {:?}", job);

  Ok(())
}
