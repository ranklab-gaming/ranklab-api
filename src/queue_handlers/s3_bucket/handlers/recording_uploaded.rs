use crate::data_types::MediaState;
use crate::fairings::sqs::QueueHandlerError;
use crate::models::{Recording, RecordingChangeset};
use crate::queue_handlers::s3_bucket::Record;
use crate::queue_handlers::S3BucketHandler;
use diesel::prelude::*;

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
  } else if file.ends_with(".jpg") {
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
