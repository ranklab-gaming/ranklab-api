use crate::data_types::MediaState;
use crate::fairings::sqs::QueueHandlerError;
use crate::models::{Avatar, AvatarChangeset};
use crate::queue_handlers::UploadsHandler;
use diesel::prelude::*;

pub async fn handle_avatar_processed(
  handler: &UploadsHandler,
  key: String,
  original_key: String,
) -> Result<(), QueueHandlerError> {
  let avatar: Avatar = handler
    .db_conn
    .run(move |conn| Avatar::find_by_image_key(&original_key).first::<Avatar>(conn))
    .await?;

  handler
    .db_conn
    .run::<_, diesel::result::QueryResult<_>>(move |conn| {
      diesel::update(&avatar)
        .set(
          AvatarChangeset::default()
            .state(MediaState::Processed)
            .processed_image_key(Some(key)),
        )
        .execute(conn)
    })
    .await
    .map_err(QueueHandlerError::from)?;

  Ok(())
}
