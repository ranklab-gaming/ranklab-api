use crate::config::Config;
use crate::data_types::MediaState;
use crate::games::GameId;
use crate::guards::{Auth, DbConn, Jwt, S3};
use crate::models::{
  Recording, RecordingChangeset, RecordingMetadata, RecordingWithCommentCount, User,
};
use crate::pagination::{Paginate, PaginatedResult};
use crate::response::{MutationResponse, QueryResponse, Response, StatusResponse};
use crate::schema::recordings;
use crate::views::{RecordingView, RecordingWithCommentCountView};
use crate::{aws, games};
use diesel::prelude::*;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;
use rocket_okapi::openapi;
use rusoto_core::{HttpClient, Region};
use rusoto_credential::AwsCredentials;
use rusoto_s3::util::PreSignedRequest;
use rusoto_s3::{DeleteObjectRequest, PutObjectRequest, S3 as RusotoS3};
use rusoto_sqs::{Sqs, SqsClient};
use schemars::JsonSchema;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;
use validator::Validate;

#[derive(Deserialize, JsonSchema, Validate)]
#[validate(schema(function = "validate_recording"))]
pub struct CreateRecordingRequest {
  #[validate(length(min = 1))]
  title: String,
  skill_level: i16,
  game_id: GameId,
  #[validate]
  metadata: RecordingMetadata,
  notes: String,
}

fn validate_recording(
  recording: &CreateRecordingRequest,
) -> Result<(), validator::ValidationError> {
  let game = games::find(&recording.game_id.to_string()).unwrap();

  if !game
    .skill_levels
    .iter()
    .any(|skill_level| skill_level.value == recording.skill_level as u8)
  {
    return Err(validator::ValidationError::new("invalid"));
  }

  Ok(())
}

#[derive(FromForm, JsonSchema)]
pub struct ListParams {
  page: Option<i64>,
  only_own: Option<bool>,
}

#[openapi(tag = "Ranklab")]
#[get("/recordings?<params..>")]
pub async fn list(
  auth: Auth<Jwt>,
  db_conn: DbConn,
  params: ListParams,
) -> QueryResponse<PaginatedResult<RecordingWithCommentCountView>> {
  let user = auth.into_user();
  let user_id = user.id;
  let page = params.page.unwrap_or(1);

  let recordings = if params.only_own.unwrap_or(false) {
    db_conn
      .run(move |conn| {
        Recording::filter_for_user(&user_id)
          .paginate(page)
          .load_and_count_pages::<RecordingWithCommentCount>(conn)
          .unwrap()
      })
      .await
  } else {
    db_conn
      .run(move |conn| {
        Recording::filter_by_game_id(&user.game_id)
          .paginate(page)
          .load_and_count_pages::<RecordingWithCommentCount>(conn)
          .unwrap()
      })
      .await
  };

  let user_ids = recordings
    .records
    .clone()
    .into_iter()
    .map(|recording| recording.recording.user_id)
    .collect::<HashSet<_>>()
    .into_iter()
    .collect::<Vec<_>>();

  let users = db_conn
    .run(move |conn| {
      User::filter_by_ids(user_ids)
        .load::<crate::models::User>(conn)
        .unwrap()
    })
    .await;

  let recording_views = recordings
    .records
    .clone()
    .into_iter()
    .map(|recording| {
      let user = users
        .clone()
        .into_iter()
        .find(|user| user.id == recording.recording.user_id)
        .unwrap();

      RecordingWithCommentCountView::new(recording, None, None, Some(user))
    })
    .collect::<Vec<RecordingWithCommentCountView>>();

  Response::success(recordings.records(recording_views))
}

#[openapi(tag = "Ranklab")]
#[post("/recordings", data = "<recording>")]
pub async fn create(
  config: &State<Config>,
  db_conn: DbConn,
  auth: Auth<Jwt>,
  recording: Json<CreateRecordingRequest>,
) -> MutationResponse<RecordingView> {
  if let Err(errors) = recording.validate() {
    return Response::validation_error(errors);
  }

  let metadata = recording.metadata.clone();
  let is_overwatch = metadata.is_overwatch();
  let key = Some(format!("recordings/originals/{}", Uuid::new_v4()));
  let user = auth.into_user();
  let user_id = user.id;

  let state = if is_overwatch {
    MediaState::Uploaded
  } else {
    MediaState::Created
  };

  let recording: Recording = db_conn
    .run(move |conn| {
      diesel::insert_into(recordings::table)
        .values(
          RecordingChangeset::default()
            .user_id(user_id)
            .game_id(recording.game_id.to_string())
            .title(recording.title.clone())
            .skill_level(recording.skill_level)
            .video_key(key)
            .metadata(Some(serde_json::to_value(&recording.metadata).unwrap()))
            .state(state)
            .notes(ammonia::clean(&recording.notes)),
        )
        .get_result::<Recording>(conn)
        .unwrap()
    })
    .await;

  if let Some(recorder_queue) = &config.recorder_queue {
    if is_overwatch {
      let mut builder = hyper::Client::builder();

      builder.pool_max_idle_per_host(0);

      let client = SqsClient::new_with(
        HttpClient::from_builder(builder, hyper_tls::HttpsConnector::new()),
        aws::CredentialsProvider::new(
          config.aws_access_key_id.clone(),
          config.aws_secret_key.clone(),
        ),
        Region::EuWest2,
      );

      let mut message = serde_json::to_value(&recording).unwrap();

      if let Some(instance_id) = &config.instance_id {
        message["instance_id"] = serde_json::Value::String(instance_id.clone());
      }

      let request = rusoto_sqs::SendMessageRequest {
        message_body: message.to_string(),
        queue_url: recorder_queue.clone(),
        ..Default::default()
      };

      client.send_message(request).await.unwrap();
    }
  }

  let url = recording
    .video_key
    .as_ref()
    .map(|video_key| create_upload_url(config, video_key));

  Response::success(RecordingView::new(
    recording,
    url,
    config.instance_id.clone(),
    Some(user),
  ))
}

#[openapi(tag = "Ranklab")]
#[get("/recordings/<id>")]
pub async fn get(
  id: Uuid,
  auth: Auth<Jwt>,
  db_conn: DbConn,
  config: &State<Config>,
) -> QueryResponse<RecordingView> {
  let user = auth.into_user();
  let game_id = user.game_id.clone();

  let recording: Recording = db_conn
    .run(move |conn| Recording::find_for_game(&game_id, &id).first::<Recording>(conn))
    .await?;

  let recording_user_id = recording.user_id;

  let recording_user = db_conn
    .run(move |conn| User::find_by_id(&recording_user_id).first::<User>(conn))
    .await?;

  let url = recording
    .video_key
    .as_ref()
    .map(|video_key| create_upload_url(config, video_key));

  Response::success(RecordingView::new(
    recording,
    url,
    None,
    Some(recording_user),
  ))
}

#[openapi(tag = "Ranklab")]
#[delete("/recordings/<id>")]
pub async fn delete(
  id: Uuid,
  auth: Auth<Jwt>,
  db_conn: DbConn,
  config: &State<Config>,
  s3: S3,
) -> MutationResponse<StatusResponse> {
  let user_id = auth.into_user().id;
  let s3 = s3.into_inner();

  let recording: Recording = db_conn
    .run(move |conn| Recording::find_for_user(&user_id, &id).first::<Recording>(conn))
    .await?;

  if let Some(video_key) = &recording.video_key {
    let req = DeleteObjectRequest {
      bucket: config.s3_bucket.to_owned(),
      key: video_key.clone(),
      ..Default::default()
    };

    s3.delete_object(req).await.unwrap();
  }

  if let Some(thumbnail_key) = &recording.thumbnail_key {
    let req = DeleteObjectRequest {
      bucket: config.s3_bucket.to_owned(),
      key: thumbnail_key.clone(),
      ..Default::default()
    };

    s3.delete_object(req).await.unwrap();
  }

  if let Some(processed_video_key) = &recording.processed_video_key {
    let req = DeleteObjectRequest {
      bucket: config.s3_bucket.to_owned(),
      key: processed_video_key.clone(),
      ..Default::default()
    };

    s3.delete_object(req).await.unwrap();
  }

  db_conn
    .run(move |conn| diesel::delete(&recording).execute(conn))
    .await?;

  Response::status(Status::NoContent)
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
