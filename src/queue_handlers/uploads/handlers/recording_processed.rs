use crate::data_types::MediaState;
use crate::emails::{Email, Recipient};
use crate::models::{Recording, RecordingChangeset, User};
use crate::queue_handlers::UploadsHandler;
use anyhow::Result;
use diesel::prelude::*;
use serde_json::json;

pub async fn handle_recording_processed(
  handler: &UploadsHandler,
  key: String,
  original_key: String,
) -> Result<()> {
  let recording = handler
    .db_conn
    .run(move |conn| Recording::find_by_video_key(&original_key).first::<Recording>(conn))
    .await?;

  if key.contains("_720p") {
    let user_id = recording.user_id;
    let recording_id = recording.id;

    handler
      .db_conn
      .run::<_, QueryResult<_>>(move |conn| {
        diesel::update(&recording)
          .set(
            RecordingChangeset::default()
              .state(MediaState::Processed)
              .processed_video_key(Some(key)),
          )
          .execute(conn)
      })
      .await?;

    let user = handler
      .db_conn
      .run(move |conn| User::find_by_id(&user_id).first::<User>(conn))
      .await?;

    let email = Email::new(
      &handler.config,
      "notification".to_owned(),
      json!({
        "subject": "Your VOD is ready!",
        "title": "Your VOD has been processed and is now ready to be reviewed.",
        "body": "You can follow the link below to view it.",
        "cta" : "View VOD",
        "cta_url" : format!("{}/recordings/{}", handler.config.web_host, recording_id),
      }),
      vec![Recipient::new(
        user.email.clone(),
        json!({
          "name": user.name.clone()
        }),
      )],
    );

    email
      .deliver()
      .await
      .map_err(|e| anyhow::anyhow!("Failed to send VOD processed email: {}", e))?;
  } else if key.contains("_thumbnail") {
    handler
      .db_conn
      .run::<_, QueryResult<_>>(move |conn| {
        diesel::update(&recording)
          .set(RecordingChangeset::default().thumbnail_key(Some(key)))
          .execute(conn)
      })
      .await?;
  }

  Ok(())
}
