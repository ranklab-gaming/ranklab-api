use crate::guards::{Auth, DbConn, Jwt};
use crate::models::{Coach, Player};
use crate::response::{QueryResponse, Response};
use crate::views::CoachView;
use diesel::prelude::*;
use rocket_okapi::openapi;

#[openapi(tag = "Ranklab")]
#[get("/player/coaches")]
pub async fn list(auth: Auth<Jwt<Player>>, db_conn: DbConn) -> QueryResponse<Vec<CoachView>> {
  let coaches: Vec<Coach> = db_conn
    .run(move |conn| Coach::filter_by_game_id(&auth.into_deep_inner().game_id).load(conn))
    .await?;

  Response::success(coaches.into_iter().map(Into::into).collect())
}
