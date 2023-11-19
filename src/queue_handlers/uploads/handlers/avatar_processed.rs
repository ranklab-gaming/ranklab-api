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

  let user_id = avatar.user_id;

  let previous_avatar: Option<Avatar> = handler
    .db_conn
    .run(move |conn| {
      Avatar::find_for_user(&user_id)
        .first::<Avatar>(conn)
        .optional()
    })
    .await?;

  let previous_avatar_id = previous_avatar.as_ref().map(|avatar| avatar.id);

  handler
    .db_conn
    .run::<_, diesel::result::QueryResult<_>>(move |conn| {
      conn.transaction(|conn| {
        if let Some(previous_avatar_id) = previous_avatar_id {
          diesel::delete(Avatar::find_by_id(&previous_avatar_id)).execute(conn)?;
        }

        diesel::update(&avatar)
          .set(
            AvatarChangeset::default()
              .state(MediaState::Processed)
              .processed_image_key(Some(key)),
          )
          .execute(conn)?;

        Ok(())
      })
    })
    .await
    .map_err(QueueHandlerError::from)?;

  if let Some(previous_avatar) = previous_avatar {
    handler.delete_upload(previous_avatar.image_key).await?;

    handler
      .delete_upload(
        previous_avatar
          .processed_image_key
          .ok_or_else(|| anyhow::anyhow!("Previous avatar has no processed image key"))?,
      )
      .await?;
  }

  Ok(())
}
