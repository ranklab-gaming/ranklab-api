use crate::aws::ConfigCredentialsProvider;
use crate::data_types::MediaState;
use crate::models::{Recording, RecordingChangeset};
use crate::queue_handlers::UploadsHandler;
use anyhow::Result;
use diesel::prelude::*;
use hyper_tls::HttpsConnector;
use rusoto_core::HttpClient;
use rusoto_rekognition::{
  NotificationChannel, Rekognition, RekognitionClient, S3Object, StartContentModerationRequest,
  Video,
};
use rusoto_signature::Region;

pub async fn handle_recording_uploaded(handler: &UploadsHandler, key: String) -> Result<()> {
  let config = &handler.config;
  let video_key = key.clone();

  let rekognition = RekognitionClient::new_with(
    HttpClient::from_connector(HttpsConnector::new()),
    ConfigCredentialsProvider::new(config.clone()),
    Region::EuWest2,
  );

  handler
    .db_conn
    .run::<_, QueryResult<_>>(move |conn| {
      diesel::update(Recording::find_by_video_key(&video_key))
        .set(RecordingChangeset::default().state(MediaState::Uploaded))
        .execute(conn)
    })
    .await?;

  rekognition
    .start_content_moderation(StartContentModerationRequest {
      video: Video {
        s3_object: Some(S3Object {
          bucket: Some(config.uploads_bucket.clone()),
          name: Some(key.clone()),
          ..Default::default()
        }),
      },
      notification_channel: Some(NotificationChannel {
        role_arn: config.rekognition_role_arn.clone(),
        sns_topic_arn: config.rekognition_topic_arn.clone(),
      }),
      ..Default::default()
    })
    .await?;

  Ok(())
}
