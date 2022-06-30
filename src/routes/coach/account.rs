use crate::data_types::UserGame;
use crate::guards::{Auth, Auth0Management, DbConn};
use crate::models::{Coach, CoachChangeset};
use crate::response::{MutationResponse, Response};
use crate::views::CoachView;
use diesel::prelude::*;
use rocket::serde::json::Json;
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde;
use serde::Deserialize;

#[derive(Deserialize, JsonSchema)]
#[schemars(rename = "CoachUpdateAccountRequest")]
pub struct UpdateAccountRequest {
  #[validate(length(min = 2))]
  name: String,
  #[validate(email)]
  email: String,
  #[validate(length(min = 1))]
  games: Vec<UserGame>,
  #[validate(length(min = 1))]
  bio: String,
}

#[openapi(tag = "Ranklab")]
#[put("/coach/account", data = "<account>")]
pub async fn update(
  account: Json<UpdateAccountRequest>,
  auth: Auth<Coach>,
  db_conn: DbConn,
  auth0_management: Auth0Management,
) -> MutationResponse<CoachView> {
  let coach = auth.0.clone();

  let coach: CoachView = db_conn
    .run(move |conn| {
      diesel::update(&coach)
        .set(
          CoachChangeset::default()
            .email(account.email.clone())
            .name(account.name.clone())
            .bio(account.bio.clone())
            .games(account.games.clone().into_iter().map(|g| Some(g)).collect()),
        )
        .get_result::<Coach>(conn)
        .unwrap()
    })
    .await
    .into();

  auth0_management
    .0
    .update_user(&auth.0.auth0_id, &coach.email)
    .await
    .unwrap();

  Response::success(coach)
}
