use crate::config::Config;
use crate::guards::{Auth, DbConn, Jwt};
use crate::models::{Audio, AudioChangeset, Coach, Review};
use crate::response::{MutationResponse, QueryResponse, Response, StatusResponse};
use crate::schema::audios;
use crate::views::AudioView;
use diesel::prelude::*;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;
use rocket_okapi::openapi;
use rusoto_core::Region;
use rusoto_credential::AwsCredentials;
use rusoto_s3::util::PreSignedRequest;
use rusoto_s3::PutObjectRequest;
use schemars::JsonSchema;
use serde::Deserialize;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Deserialize, JsonSchema)]
pub struct CreateAudioRequest {
  pub review_id: Uuid,
}

#[openapi(tag = "Ranklab")]
#[get("/coach/audios/<id>")]
pub async fn get(id: Uuid, db_conn: DbConn) -> QueryResponse<AudioView> {
  let audio = db_conn
    .run(move |conn| Audio::find_by_id(&id).first::<Audio>(conn))
    .await?;

  Response::success(AudioView::new(audio, None, None))
}

#[openapi(tag = "Ranklab")]
#[post("/coach/audios", data = "<audio>")]
pub async fn create(
  config: &State<Config>,
  db_conn: DbConn,
  auth: Auth<Jwt<Coach>>,
  audio: Json<CreateAudioRequest>,
) -> MutationResponse<AudioView> {
  let key = format!("audios/originals/{}", Uuid::new_v4());
  let coach = auth.into_deep_inner();

  let review = db_conn
    .run(move |conn| {
      Review::find_for_coach(&audio.review_id, &coach.id)
        .first::<Review>(conn)
        .unwrap()
    })
    .await;

  let audio: Audio = db_conn
    .run(move |conn| {
      diesel::insert_into(audios::table)
        .values(
          AudioChangeset::default()
            .audio_key(key)
            .review_id(review.id),
        )
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
#[delete("/coach/audios/<id>")]
pub async fn delete(
  db_conn: DbConn,
  _auth: Auth<Jwt<Coach>>,
  id: Uuid,
) -> MutationResponse<StatusResponse> {
  db_conn
    .run(move |conn| {
      diesel::delete(Audio::find_by_id(&id))
        .execute(conn)
        .unwrap()
    })
    .await;

  Response::status(Status::NoContent)
}
