use crate::aws;
use crate::data_types::MediaState;
use crate::fairings::sqs::QueueHandlerError;
use crate::models::{Avatar, AvatarChangeset};
use crate::queue_handlers::UploadsHandler;
use diesel::prelude::*;
use hyper_tls::HttpsConnector;
use rusoto_core::HttpClient;
use rusoto_lambda::{Lambda, LambdaClient};
use rusoto_rekognition::Rekognition;
use rusoto_s3::S3;
use rusoto_signature::Region;

pub async fn handle_avatar_uploaded(
  handler: &UploadsHandler,
  key: String,
  message: String,
) -> Result<(), QueueHandlerError> {
  let config = &handler.config;
  let builder = hyper::Client::builder();

  let lambda = LambdaClient::new_with(
    HttpClient::from_builder(builder.clone(), HttpsConnector::new()),
    aws::CredentialsProvider::new(
      config.aws_access_key_id.clone(),
      config.aws_secret_key.clone(),
    ),
    Region::EuWest2,
  );

  let rekognition = rusoto_rekognition::RekognitionClient::new_with(
    HttpClient::from_builder(builder, HttpsConnector::new()),
    aws::CredentialsProvider::new(
      config.aws_access_key_id.clone(),
      config.aws_secret_key.clone(),
    ),
    Region::EuWest2,
  );

  let image_key = key.clone();

  let object = handler
    .client
    .head_object(rusoto_s3::HeadObjectRequest {
      bucket: config.uploads_bucket.clone(),
      key: key.clone(),
      ..Default::default()
    })
    .await
    .map_err(anyhow::Error::from)?;

  let avatar: Avatar = handler
    .db_conn
    .run(move |conn| Avatar::find_by_image_key(&image_key).first::<Avatar>(conn))
    .await?;

  let avatar_to_update = avatar.clone();

  handler
    .db_conn
    .run::<_, diesel::result::QueryResult<_>>(move |conn| {
      diesel::update(&avatar_to_update)
        .set(AvatarChangeset::default().state(MediaState::Uploaded))
        .execute(conn)
    })
    .await
    .map_err(QueueHandlerError::from)?;

  let content_type = object
    .content_type
    .ok_or_else(|| anyhow::anyhow!("No content type found for object"))?;

  if !["image/jpeg", "image/png"].contains(&content_type.as_str()) {
    return handler.delete_upload(key).await;
  }

  let moderation_labels = rekognition
    .detect_moderation_labels(rusoto_rekognition::DetectModerationLabelsRequest {
      image: rusoto_rekognition::Image {
        s3_object: Some(rusoto_rekognition::S3Object {
          bucket: Some(config.uploads_bucket.clone()),
          name: Some(key.clone()),
          ..Default::default()
        }),
        ..Default::default()
      },
      ..Default::default()
    })
    .await
    .map_err(anyhow::Error::from)?;

  if let Some(moderation_labels) = moderation_labels.moderation_labels {
    if !moderation_labels.is_empty() {
      return handler.delete_upload(key).await;
    }
  }

  let avatar_processor_lambda_arn = config.avatar_processor_lambda_arn.clone();

  lambda
    .invoke(rusoto_lambda::InvocationRequest {
      function_name: avatar_processor_lambda_arn,
      invocation_type: Some("Event".to_owned()),
      payload: Some(message.into_bytes().into()),
      ..Default::default()
    })
    .await
    .map_err(anyhow::Error::from)?;

  Ok(())
}
