use crate::db::DbConn;
use crate::guards::Auth;
use crate::models::{Coach, Review};
use crate::response::{QueryResponse, Response};
use diesel::prelude::*;
use rocket_okapi::openapi;
use uuid::Uuid;

#[openapi(tag = "Ranklab")]
#[get("/coach/reviews")]
pub async fn list(auth: Auth<Coach>, db_conn: DbConn) -> QueryResponse<Vec<Review>> {
  let reviews = db_conn
    .run(move |conn| {
      use crate::schema::reviews::dsl::*;
      use diesel::dsl::sql;
      use diesel::pg::Pg;
      use diesel::sql_types::Bool;

      let mut games_expression: Box<dyn BoxableExpression<reviews, Pg, SqlType = Bool>> =
        Box::new(sql::<Bool>("false"));

      for game in auth.0.games.into_iter() {
        games_expression = Box::new(
          games_expression.or(
            game_id
              .eq(game.game_id)
              .and(skill_level.lt(game.skill_level as i16)),
          ),
        );
      }

      reviews
        .filter(
          coach_id
            .eq(auth.0.id)
            .or(coach_id.eq::<Option<Uuid>>(None).and(games_expression)),
        )
        .load(conn)
        .unwrap()
    })
    .await;

  Response::success(reviews)
}

#[openapi(tag = "Ranklab")]
#[get("/coach/reviews/<id>")]
pub async fn get(id: Uuid, auth: Auth<Coach>, db_conn: DbConn) -> QueryResponse<Review> {
  let review = db_conn
    .run(move |conn| {
      use crate::schema::reviews::dsl::{coach_id, id as review_id, reviews};
      reviews
        .filter(coach_id.eq(auth.0.id).and(review_id.eq(id)))
        .first(conn)
    })
    .await?;

  Response::success(review)
}
