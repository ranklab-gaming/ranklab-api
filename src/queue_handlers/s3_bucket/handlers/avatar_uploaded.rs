use crate::data_types::MediaState;
use crate::fairings::sqs::QueueHandlerError;
use crate::models::{Avatar, AvatarChangeset};
use crate::queue_handlers::s3_bucket::Record;
use crate::queue_handlers::S3BucketHandler;
use diesel::prelude::*;

pub async fn handle_avatar_uploaded(
  handler: &S3BucketHandler,
  record: &Record,
  folder: &str,
  file: &str,
) -> Result<(), QueueHandlerError> {
  let image_key = format!(
    "avatars/originals/{}",
    file.split('.').collect::<Vec<_>>()[0]
  );

  let avatar: Avatar = handler
    .db_conn
    .run(move |conn| Avatar::find_by_image_key(&image_key).first::<Avatar>(conn))
    .await?;

  if folder == "originals" {
    handler
      .db_conn
      .run::<_, diesel::result::QueryResult<_>>(move |conn| {
        diesel::update(&avatar)
          .set(AvatarChangeset::default().state(MediaState::Uploaded))
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

  handler
    .db_conn
    .run::<_, diesel::result::QueryResult<_>>(move |conn| {
      diesel::update(&avatar)
        .set(
          AvatarChangeset::default()
            .state(MediaState::Processed)
            .processed_image_key(processed_image_key),
        )
        .get_result::<Avatar>(conn)
    })
    .await
    .map_err(QueueHandlerError::from)?;

  Ok(())
}
