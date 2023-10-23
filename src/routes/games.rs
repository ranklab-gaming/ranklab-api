use crate::guards::{Auth, DbConn, Jwt};
use crate::models::{Following, FollowingChangeset};
use crate::response::{MutationResponse, QueryResponse, Response};
use crate::schema::followings;
use crate::views::GameView;
use diesel::prelude::*;
use rocket::serde::json::Json;
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde::Deserialize;

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

#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateGameRequest {
  followed: bool,
}

#[openapi(tag = "Ranklab")]
#[put("/games/<id>", data = "<request>")]
pub async fn update(
  id: String,
  request: Json<UpdateGameRequest>,
  auth: Auth<Jwt>,
  db_conn: DbConn,
) -> MutationResponse<GameView> {
  let user = auth.into_user();
  let game = crate::games::find(&id).unwrap();

  let following = db_conn
    .run(move |conn| {
      let following = Following::find_for_user_and_game(&user.id, &id)
        .get_result::<Following>(conn)
        .optional()
        .unwrap();

      match (request.followed, &following) {
        (false, Some(following)) => {
          diesel::delete(&following).execute(conn).unwrap();
          None
        }
        (true, None) => diesel::insert_into(followings::table)
          .values(
            FollowingChangeset::default()
              .user_id(user.id)
              .game_id(id.to_string()),
          )
          .get_result::<Following>(conn)
          .ok(),
        _ => following,
      }
    })
    .await;

  Response::success(GameView::new(game, following.is_some()))
}
