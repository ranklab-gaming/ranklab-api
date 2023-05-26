use std::collections::HashMap;

use crate::config::Config;
use crate::data_types::MediaState;
use crate::games;
use crate::guards::{Auth, DbConn, Jwt};
use crate::models::{Player, Recording, RecordingChangeset};
use crate::response::{MutationResponse, QueryResponse, Response};
use crate::schema::recordings;
use crate::views::RecordingView;
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
use uuid::Uuid;
use validator::Validate;

#[derive(Deserialize, JsonSchema, Validate)]
pub struct CreateRecordingRequest {
  #[validate(length(min = 1))]
  title: String,
  skill_level: i16,
  #[validate(length(min = 1), custom = "games::validate_id")]
  game_id: String,
  metadata: Option<serde_json::Value>,
}

#[openapi(tag = "Ranklab")]
#[get("/player/recordings")]
pub async fn list(auth: Auth<Jwt<Player>>, db_conn: DbConn) -> QueryResponse<Vec<RecordingView>> {
  let recordings: Vec<Recording> = db_conn
    .run(move |conn| {
      Recording::filter_for_player(&auth.into_deep_inner().id).load::<Recording>(conn)
    })
    .await?;

  let recording_views: Vec<RecordingView> = recordings.into_iter().map(Into::into).collect();

  Response::success(recording_views)
}

#[openapi(tag = "Ranklab")]
#[post("/player/recordings", data = "<recording>")]
pub async fn create(
  config: &State<Config>,
  db_conn: DbConn,
  auth: Auth<Jwt<Player>>,
  recording: Json<CreateRecordingRequest>,
) -> MutationResponse<RecordingView> {
  if let Err(errors) = recording.validate() {
    return Response::validation_error(errors);
  }

  let game = games::find(&recording.game_id).unwrap();

  if !game
    .skill_levels
    .iter()
    .any(|skill_level| skill_level.value == recording.skill_level as u8)
  {
    return Response::mutation_error(Status::UnprocessableEntity);
  }

  let key = match recording.metadata {
    Some(_) => None,
    None => Some(format!("recordings/originals/{}", Uuid::new_v4())),
  };

  let state = match recording.metadata {
    Some(_) => MediaState::Processed,
    None => MediaState::Created,
  };

  let recording: Recording = db_conn
    .run(move |conn| {
      diesel::insert_into(recordings::table)
        .values(
          RecordingChangeset::default()
            .player_id(auth.into_deep_inner().id)
            .game_id(recording.game_id.clone())
            .title(recording.title.clone())
            .skill_level(recording.skill_level)
            .video_key(key)
            .metadata(recording.metadata.clone())
            .state(state),
        )
        .get_result::<Recording>(conn)
        .unwrap()
    })
    .await;

  let url = recording
    .video_key
    .as_ref()
    .map(|video_key| create_upload_url(config, video_key));

  Response::success(RecordingView::new(
    recording,
    url,
    config.instance_id.clone(),
  ))
}

#[openapi(tag = "Ranklab")]
#[get("/player/recordings/<id>")]
pub async fn get(
  id: Uuid,
  auth: Auth<Jwt<Player>>,
  db_conn: DbConn,
  config: &State<Config>,
) -> QueryResponse<RecordingView> {
  let recording: Recording = db_conn
    .run(move |conn| {
      Recording::find_for_player(&id, &auth.into_deep_inner().id).first::<Recording>(conn)
    })
    .await?;

  let url = recording
    .video_key
    .as_ref()
    .map(|video_key| create_upload_url(config, video_key));

  Response::success(RecordingView::new(recording, url, None))
}

fn create_upload_url(config: &Config, recording_video_key: &String) -> String {
  let mut metadata = HashMap::new();

  if let Some(instance_id) = config.instance_id.as_ref() {
    metadata.insert("instance-id".to_string(), instance_id.to_string());
  }

  let req = PutObjectRequest {
    bucket: config.s3_bucket.to_owned(),
    key: recording_video_key.to_owned(),
    acl: Some("public-read".to_string()),
    metadata: Some(metadata),
    ..Default::default()
  };

  req.get_presigned_url(
    &Region::EuWest2,
    &AwsCredentials::new(
      &config.aws_access_key_id,
      &config.aws_secret_key,
      None,
      None,
    ),
    &Default::default(),
  )
}
