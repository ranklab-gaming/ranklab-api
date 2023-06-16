use crate::config::Config;
use crate::guards::{Auth, DbConn, Jwt, S3};
use crate::models::{Audio, AudioChangeset};
use crate::response::{MutationResponse, QueryResponse, Response, StatusResponse};
use crate::schema::audios;
use crate::views::AudioView;
use diesel::prelude::*;
use rocket::http::Status;
use rocket::State;
use rocket_okapi::openapi;
use rusoto_core::Region;
use rusoto_credential::AwsCredentials;
use rusoto_s3::util::PreSignedRequest;
use rusoto_s3::{DeleteObjectRequest, PutObjectRequest, S3 as RusotoS3};
use std::collections::HashMap;
use uuid::Uuid;

#[openapi(tag = "Ranklab")]
#[get("/audios/<id>")]
pub async fn get(auth: Auth<Jwt>, id: Uuid, db_conn: DbConn) -> QueryResponse<AudioView> {
  let user = auth.into_user();

  let audio = db_conn
    .run(move |conn| Audio::find_for_user(&user.id, &id).first::<Audio>(conn))
    .await?;

  Response::success(AudioView::new(audio, None, None))
}

#[openapi(tag = "Ranklab")]
#[post("/audios")]
pub async fn create(
  config: &State<Config>,
  db_conn: DbConn,
  auth: Auth<Jwt>,
) -> MutationResponse<AudioView> {
  let key = format!("audios/originals/{}", Uuid::new_v4());
  let user = auth.into_user();
  let user_id = user.id;

  let audio: Audio = db_conn
    .run(move |conn| {
      diesel::insert_into(audios::table)
        .values(AudioChangeset::default().audio_key(key).user_id(user_id))
        .get_result::<Audio>(conn)
        .unwrap()
    })
    .await;

  let mut metadata = HashMap::new();

  if let Some(instance_id) = config.instance_id.as_ref() {
    metadata.insert("instance-id".to_string(), instance_id.to_string());
  }

  let req = PutObjectRequest {
    bucket: config.s3_bucket.to_owned(),
    key: audio.audio_key.to_owned(),
    acl: Some("public-read".to_string()),
    metadata: Some(metadata),
    ..Default::default()
  };

  let url = req.get_presigned_url(
    &Region::EuWest2,
    &AwsCredentials::new(
      &config.aws_access_key_id,
      &config.aws_secret_key,
      None,
      None,
    ),
    &Default::default(),
  );

  Response::success(AudioView::new(audio, Some(url), config.instance_id.clone()))
}

#[openapi(tag = "Ranklab")]
#[delete("/audios/<id>")]
pub async fn delete(
  db_conn: DbConn,
  auth: Auth<Jwt>,
  id: Uuid,
  config: &State<Config>,
  s3: S3,
) -> MutationResponse<StatusResponse> {
  let user = auth.into_user();
  let s3 = s3.into_inner();

  let audio: Audio = db_conn
    .run(move |conn| Audio::find_for_user(&user.id, &id).first::<Audio>(conn))
    .await?;

  let req = DeleteObjectRequest {
    bucket: config.s3_bucket.to_owned(),
    key: audio.audio_key.clone(),
    ..Default::default()
  };

  s3.delete_object(req).await.unwrap();

  if let Some(processed_audio_key) = &audio.processed_audio_key {
    let req = DeleteObjectRequest {
      bucket: config.s3_bucket.to_owned(),
      key: processed_audio_key.clone(),
      ..Default::default()
    };

    s3.delete_object(req).await.unwrap();
  }

  db_conn
    .run(move |conn| diesel::delete(&audio).execute(conn).unwrap())
    .await;

  Response::status(Status::NoContent)
}
