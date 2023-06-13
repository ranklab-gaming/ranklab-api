use crate::guards::{Auth, DbConn, Jwt};
use crate::models::{Avatar, Coach, Player};
use crate::response::{QueryResponse, Response};
use crate::views::CoachView;
use diesel::prelude::*;
use rocket_okapi::openapi;
use uuid::Uuid;

#[openapi(tag = "Ranklab")]
#[get("/player/coaches")]
pub async fn list(auth: Auth<Jwt<Player>>, db_conn: DbConn) -> QueryResponse<Vec<CoachView>> {
  let coaches: Vec<Coach> = db_conn
    .run(move |conn| Coach::filter_by_game_id(&auth.into_deep_inner().game_id).load(conn))
    .await?;

  let avatar_ids = coaches
    .iter()
    .filter_map(|coach| coach.avatar_id)
    .collect::<Vec<Uuid>>();

  let avatars: Vec<Avatar> = db_conn
    .run(move |conn| Avatar::filter_by_ids(avatar_ids).load::<Avatar>(conn))
    .await?;

  let coach_views: Vec<CoachView> = coaches
    .clone()
    .into_iter()
    .map(|coach| {
      let avatar_id = coach.avatar_id;

      CoachView::new(
        coach,
        None,
        avatars
          .iter()
          .find(|avatar| Some(avatar.id) == avatar_id)
          .cloned(),
      )
    })
    .collect();

  Response::success(coach_views)
}

#[openapi(tag = "Ranklab")]
#[get("/player/coaches/<slug>")]
pub async fn get(db_conn: DbConn, slug: String) -> QueryResponse<CoachView> {
  let coach = db_conn
    .run(move |conn| Coach::find_by_slug(&slug).first::<Coach>(conn))
    .await?;

  let avatar = match coach.avatar_id {
    Some(avatar_id) => db_conn
      .run(move |conn| Avatar::find_processed_by_id(&avatar_id).get_result::<Avatar>(conn))
      .await
      .ok(),
    None => None,
  };

  Response::success(CoachView::new(coach, None, avatar))
}
