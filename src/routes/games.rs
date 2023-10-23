use crate::guards::{Auth, DbConn, Jwt};
use crate::models::Following;
use crate::response::{QueryResponse, Response};
use crate::views::GameView;
use diesel::prelude::*;
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde::Deserialize;
use validator::Validate;

#[derive(Deserialize, JsonSchema, Validate)]
pub struct CreateGameRequest {
  #[validate(email)]
  email: String,
  #[validate(length(min = 1))]
  name: String,
}

#[openapi(tag = "Ranklab")]
#[get("/games")]
pub async fn list(auth: Auth<Option<Jwt>>, db_conn: DbConn) -> QueryResponse<Vec<GameView>> {
  let user = auth.into_user();

  let games = crate::games::all().iter().collect::<Vec<_>>();

  let followings: Vec<Following> = match user {
    Some(user) => {
      db_conn
        .run(move |conn| {
          Following::filter_for_user(&user.id)
            .load::<Following>(conn)
            .unwrap()
        })
        .await
    }
    None => vec![],
  };

  let game_views = games
    .clone()
    .into_iter()
    .map(|game| {
      let followed = followings
        .iter()
        .any(|following| following.game_id == game.id.to_string());

      GameView::new(game, followed)
    })
    .collect::<Vec<_>>();

  Response::success(game_views)
}
