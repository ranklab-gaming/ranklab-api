use crate::data_types::MediaState;
use crate::fairings::sqs::QueueHandlerError;
use crate::models::{Recording, RecordingChangeset};
use crate::queue_handlers::UploadsHandler;
use diesel::prelude::*;

pub async fn handle_recording_processed(
  handler: &UploadsHandler,
  key: String,
  original_key: String,
) -> Result<(), QueueHandlerError> {
  let recording: Recording = handler
    .db_conn
    .run(move |conn| Recording::find_by_video_key(&original_key).first::<Recording>(conn))
    .await?;

  if key.ends_with(".mp4") {
    handler
      .db_conn
      .run::<_, diesel::result::QueryResult<_>>(move |conn| {
        diesel::update(&recording)
          .set(
            RecordingChangeset::default()
              .state(MediaState::Processed)
              .processed_video_key(Some(key)),
          )
          .execute(conn)
      })
      .await
      .map_err(QueueHandlerError::from)?;
  } else if key.ends_with(".jpg") {
    handler
      .db_conn
      .run::<_, diesel::result::QueryResult<_>>(move |conn| {
        diesel::update(&recording)
          .set(RecordingChangeset::default().thumbnail_key(Some(key)))
          .execute(conn)
      })
      .await
      .map_err(QueueHandlerError::from)?;
  }

  Ok(())
}
