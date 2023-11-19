use crate::aws::ConfigCredentialsProvider;
use crate::data_types::MediaState;
use crate::models::{Avatar, AvatarChangeset};
use crate::queue_handlers::UploadsHandler;
use anyhow::Result;
use diesel::prelude::*;
use hyper_tls::HttpsConnector;
use rusoto_core::HttpClient;
use rusoto_lambda::{InvocationRequest, Lambda, LambdaClient};
use rusoto_rekognition::{
  DetectModerationLabelsRequest, Image, Rekognition, RekognitionClient, S3Object,
};
use rusoto_s3::{HeadObjectRequest, S3};
use rusoto_signature::Region;

pub async fn handle_avatar_uploaded(
  handler: &UploadsHandler,
  key: String,
  message: String,
) -> Result<()> {
  let config = &handler.config;

  let lambda = LambdaClient::new_with(
    HttpClient::from_connector(HttpsConnector::new()),
    ConfigCredentialsProvider::new(config.clone()),
    Region::EuWest2,
  );

  let rekognition = RekognitionClient::new_with(
    HttpClient::from_connector(HttpsConnector::new()),
    ConfigCredentialsProvider::new(config.clone()),
    Region::EuWest2,
  );

  let image_key = key.clone();

  let object = handler
    .client
    .head_object(HeadObjectRequest {
      bucket: config.uploads_bucket.clone(),
      key: key.clone(),
      ..Default::default()
    })
    .await?;

  let avatar = handler
    .db_conn
    .run(move |conn| Avatar::find_by_image_key(&image_key).first::<Avatar>(conn))
    .await?;

  let avatar_to_update = avatar.clone();

  handler
    .db_conn
    .run::<_, QueryResult<_>>(move |conn| {
      diesel::update(&avatar_to_update)
        .set(AvatarChangeset::default().state(MediaState::Uploaded))
        .execute(conn)
    })
    .await?;

  let content_type = object
    .content_type
    .ok_or_else(|| anyhow::anyhow!("No content type found for object"))?;

  if !["image/jpeg", "image/png"].contains(&content_type.as_str()) {
    return handler.delete_upload(key).await;
  }

  let moderation_labels = rekognition
    .detect_moderation_labels(DetectModerationLabelsRequest {
      image: Image {
        s3_object: Some(S3Object {
          bucket: Some(config.uploads_bucket.clone()),
          name: Some(key.clone()),
          ..Default::default()
        }),
        ..Default::default()
      },
      ..Default::default()
    })
    .await?;

  if let Some(moderation_labels) = moderation_labels.moderation_labels {
    if !moderation_labels.is_empty() {
      return handler.delete_upload(key).await;
    }
  }

  let avatar_processor_lambda_arn = config.avatar_processor_lambda_arn.clone();

  lambda
    .invoke(InvocationRequest {
      function_name: avatar_processor_lambda_arn,
      invocation_type: Some("Event".to_owned()),
      payload: Some(message.into_bytes().into()),
      ..Default::default()
    })
    .await?;

  Ok(())
}
