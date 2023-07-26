use crate::data_types::MediaState;
use crate::emails::{Email, Recipient};
use crate::fairings::sqs::QueueHandlerError;
use crate::models::{Recording, RecordingChangeset, RecordingMetadata, User};
use crate::queue_handlers::s3_bucket::Record;
use crate::queue_handlers::S3BucketHandler;
use diesel::prelude::*;
use serde_json::json;

pub async fn handle_recording_uploaded(
  handler: &S3BucketHandler,
  record: &Record,
  folder: &str,
  file: &str,
  _profile: &rocket::figment::Profile,
) -> Result<(), QueueHandlerError> {
  let video_key = format!(
    "recordings/originals/{}",
    file.split('_').collect::<Vec<_>>()[0]
  );

  let recording_query = Recording::find_by_video_key(&video_key);

  if folder == "originals" {
    handler
      .db_conn
      .run::<_, diesel::result::QueryResult<_>>(move |conn| {
        diesel::update(recording_query)
          .set(RecordingChangeset::default().state(MediaState::Uploaded))
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

    let recording: Recording = handler
      .db_conn
      .run(move |conn| recording_query.first::<Recording>(conn))
      .await
      .map_err(QueueHandlerError::from)?;

    let metadata = recording.metadata.clone();
    let recording_id = recording.id;
    let user_id = recording.user_id;
    let title = recording.title.clone();

    handler
      .db_conn
      .run::<_, diesel::result::QueryResult<_>>(move |conn| {
        diesel::update(&recording)
          .set(
            RecordingChangeset::default()
              .state(MediaState::Processed)
              .processed_video_key(processed_video_key),
          )
          .execute(conn)
      })
      .await
      .map_err(QueueHandlerError::from)?;

    if let Some(metadata) = metadata {
      let parsed_metadata = serde_json::from_value::<RecordingMetadata>(metadata)
        .map_err(|e| anyhow::anyhow!("Failed to parse metadata: {}", e))?;

      let user = handler
        .db_conn
        .run(move |conn| User::find_by_id(&user_id).first::<User>(conn))
        .await?;

      if parsed_metadata.is_overwatch() {
        let video_uploaded = Email::new(
          &handler.config,
          "notification".to_owned(),
          json!({
            "subject": "Your VOD has been processed",
            "title": format!("The VOD for \"{}\" has been processed", title),
            "body": "You can follow the link below to view it.",
            "cta" : "View VOD",
            "cta_url" : format!("{}/recordings/{}", handler.config.web_host, recording_id),
          }),
          vec![Recipient::new(
            user.email,
            json!({
              "name": user.name,
            }),
          )],
        );

        video_uploaded
          .deliver()
          .await
          .map_err(|e| anyhow::anyhow!("Failed to send email: {}", e))?;
      }
    }

    return Ok(());
  }

  if file.ends_with(".jpg") {
    let thumbnail_key = Some(record.s3.object.key.clone());

    handler
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
