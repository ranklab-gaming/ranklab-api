use std::collections::HashMap;

use crate::auth::Account;
use crate::config::Config;
use crate::guards::{Auth, DbConn, Jwt};
use crate::models::{Avatar, AvatarChangeset, Coach, CoachChangeset, Player, PlayerChangeset};
use crate::response::{MutationResponse, QueryResponse, Response, StatusResponse};
use crate::schema::avatars;
use crate::views::AvatarView;
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

#[derive(Deserialize, JsonSchema)]
pub struct CreateAvatarRequest {}

#[openapi(tag = "Ranklab")]
#[post("/avatars", data = "<_avatar>")]
pub async fn create(
  config: &State<Config>,
  db_conn: DbConn,
  auth: Auth<Jwt<Account>>,
  _avatar: Json<CreateAvatarRequest>,
) -> MutationResponse<AvatarView> {
  let account = auth.into_deep_inner();
  let key = format!("avatars/originals/{}", Uuid::new_v4());

  let avatar: Avatar = db_conn
    .run(move |conn| {
      diesel::insert_into(avatars::table)
        .values(AvatarChangeset::default().image_key(key))
        .get_result::<Avatar>(conn)
        .unwrap()
    })
    .await;

  let avatar_id = avatar.id;

  match account {
    Account::Coach(coach) => {
      db_conn
        .run(move |conn| {
          diesel::update(Coach::find_by_id(&coach.id))
            .set(CoachChangeset::default().avatar_id(Some(avatar_id)))
            .get_result::<Coach>(conn)
            .unwrap()
        })
        .await;
    }
    Account::Player(player) => {
      db_conn
        .run(move |conn| {
          diesel::update(Player::find_by_id(&player.id))
            .set(PlayerChangeset::default().avatar_id(Some(avatar_id)))
            .get_result::<Player>(conn)
            .unwrap()
        })
        .await;
    }
  }

  let mut metadata = HashMap::new();

  if let Some(instance_id) = config.instance_id.as_ref() {
    metadata.insert("instance-id".to_string(), instance_id.to_string());
  }

  let req = PutObjectRequest {
    bucket: config.s3_bucket.to_owned(),
    key: avatar.image_key.to_owned(),
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

  Response::success(AvatarView::new(
    avatar,
    Some(url),
    config.instance_id.clone(),
  ))
}

#[openapi(tag = "Ranklab")]
#[delete("/avatars")]
pub async fn delete(db_conn: DbConn, auth: Auth<Jwt<Account>>) -> MutationResponse<StatusResponse> {
  let account = auth.into_deep_inner();

  let avatar_id = match account {
    Account::Coach(coach) => coach.avatar_id,
    Account::Player(player) => player.avatar_id,
  };

  if let Some(avatar_id) = avatar_id {
    db_conn
      .run(move |conn| {
        diesel::delete(Avatar::find_processed_by_id(&avatar_id))
          .execute(conn)
          .unwrap()
      })
      .await;
  }

  Response::status(Status::NoContent)
}

#[openapi(tag = "Ranklab")]
#[get("/avatars/<id>")]
pub async fn get(id: Uuid, db_conn: DbConn) -> QueryResponse<AvatarView> {
  let avatar = db_conn
    .run(move |conn| Avatar::find_by_id(&id).first::<Avatar>(conn))
    .await?;

  Response::success(AvatarView::new(avatar, None, None))
}
