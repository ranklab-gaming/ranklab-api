use crate::data_types::PlayerGame;
use crate::guards::{Auth, Auth0Management, DbConn};
use crate::models::{Player, PlayerChangeset};
use crate::response::{MutationResponse, Response};
use crate::views::PlayerView;
use diesel::prelude::*;
use rocket::serde::json::Json;
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde;
use serde::Deserialize;

#[derive(Deserialize, JsonSchema)]
#[schemars(rename = "PlayerUpdateAccountRequest")]
pub struct UpdateAccountRequest {
  #[validate(length(min = 2))]
  name: String,
  #[validate(email)]
  email: String,
  #[validate(length(min = 1))]
  games: Vec<PlayerGame>,
}

#[openapi(tag = "Ranklab")]
#[put("/player/account", data = "<account>")]
pub async fn update(
  account: Json<UpdateAccountRequest>,
  auth: Auth<Player>,
  db_conn: DbConn,
  auth0_management: Auth0Management,
) -> MutationResponse<PlayerView> {
  let player = auth.0.clone();

  let player: PlayerView = db_conn
    .run(move |conn| {
      diesel::update(&player)
        .set(
          PlayerChangeset::default()
            .email(account.email.clone())
            .name(account.name.clone())
            .games(account.games.clone().into_iter().map(|g| Some(g)).collect()),
        )
        .get_result::<Player>(conn)
        .unwrap()
    })
    .await
    .into();

  auth0_management
    .0
    .update_user(&auth.0.auth0_id, &player.email)
    .await
    .unwrap();

  Response::success(player)
}
