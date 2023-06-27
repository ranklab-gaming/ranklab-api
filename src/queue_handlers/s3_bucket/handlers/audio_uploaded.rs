use crate::data_types::MediaState;
use crate::fairings::sqs::QueueHandlerError;
use crate::models::{Audio, AudioChangeset};
use crate::queue_handlers::s3_bucket::{Record, WhisperApiResponse};
use crate::queue_handlers::S3BucketHandler;
use anyhow::anyhow;
use diesel::prelude::*;
use reqwest::multipart;
use rocket::tokio::io::AsyncReadExt;
use rusoto_s3::{GetObjectRequest, S3};

pub async fn handle_audio_uploaded(
  handler: &S3BucketHandler,
  record: &Record,
  folder: &str,
  file: &str,
) -> Result<(), QueueHandlerError> {
  let audio_key = format!(
    "audios/originals/{}",
    file.split('_').collect::<Vec<_>>()[0]
  );

  let audio_query = Audio::find_by_audio_key(&audio_key);

  if folder == "originals" {
    let audio: Audio = handler
      .db_conn
      .run(move |conn| audio_query.first::<Audio>(conn))
      .await?;

    handler
      .db_conn
      .run::<_, diesel::result::QueryResult<_>>(move |conn| {
        diesel::update(&audio)
          .set(AudioChangeset::default().state(MediaState::Uploaded))
          .execute(conn)
      })
      .await
      .map_err(QueueHandlerError::from)?;

    return Ok(());
  }

  if folder != "processed" {
    return Ok(());
  }

  let processed_audio_key = Some(record.s3.object.key.clone());
  let mut transcript: Option<String> = None;

  if let Some(whisper_api_key) = &handler.config.whisper_api_key {
    let processed_audio_file = handler
      .client
      .get_object(GetObjectRequest {
        bucket: handler.config.s3_bucket.clone(),
        key: record.s3.object.key.clone(),
        ..Default::default()
      })
      .await
      .map_err(|e| QueueHandlerError::from(anyhow!(e)))?;

    let stream = processed_audio_file
      .body
      .ok_or_else(|| QueueHandlerError::from(anyhow!("no body found in s3 response")))?;

    let mut bytes = Vec::new();

    stream
      .into_async_read()
      .read_to_end(&mut bytes)
      .await
      .map_err(|e| QueueHandlerError::from(anyhow!("error reading s3 response body: {}", e)))?;

    let reqwest = reqwest::Client::new();

    let part = multipart::Part::bytes(bytes)
      .file_name("audio.mp4")
      .mime_str("audio/mp4")
      .map_err(|e| QueueHandlerError::from(anyhow!(e)))?;

    let form = multipart::Form::new()
      .part("file", part)
      .text("model", "whisper-1");

    let response = reqwest
      .post("https://api.openai.com/v1/audio/transcriptions")
      .bearer_auth(whisper_api_key.clone())
      .multipart(form)
      .send()
      .await
      .map_err(|e| QueueHandlerError::from(anyhow!(e)))?;

    let json: WhisperApiResponse = response
      .json()
      .await
      .map_err(|e| QueueHandlerError::from(anyhow!(e)))?;

    transcript = Some(json.text);
  }

  handler
    .db_conn
    .run::<_, diesel::result::QueryResult<_>>(move |conn| {
      diesel::update(audio_query)
        .set(
          AudioChangeset::default()
            .state(MediaState::Processed)
            .processed_audio_key(processed_audio_key)
            .transcript(transcript),
        )
        .execute(conn)
    })
    .await
    .map_err(QueueHandlerError::from)?;

  Ok(())
}
