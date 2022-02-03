use crate::guards::Auth;
use crate::guards::DbConn;
use crate::models::{Coach, Review};
use crate::response::{QueryResponse, Response};
use crate::views::ReviewView;
use diesel::prelude::*;
use rocket_okapi::openapi;
use schemars::JsonSchema;
use uuid::Uuid;

#[derive(FromForm, JsonSchema)]
pub struct ListReviewsQuery {
  pending: Option<bool>,
}

#[openapi(tag = "Ranklab")]
#[get("/coach/reviews?<params..>")]
pub async fn list(
  auth: Auth<Coach>,
  db_conn: DbConn,
  params: ListReviewsQuery,
) -> QueryResponse<Vec<ReviewView>> {
  let reviews: Vec<ReviewView> = db_conn
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

      let query = if params.pending.unwrap_or(false) {
        reviews
          .filter(coach_id.eq::<Option<Uuid>>(None).and(games_expression))
          .into_boxed()
      } else {
        reviews.filter(coach_id.eq(auth.0.id)).into_boxed()
      };

      query.load::<Review>(conn).unwrap()
    })
    .await
    .into_iter()
    .map(Into::into)
    .collect();

  Response::success(reviews)
}

#[openapi(tag = "Ranklab")]
#[get("/coach/reviews/<id>")]
pub async fn get(id: Uuid, auth: Auth<Coach>, db_conn: DbConn) -> QueryResponse<ReviewView> {
  let review: ReviewView = db_conn
    .run(move |conn| {
      use crate::schema::reviews::dsl::{coach_id, id as review_id, reviews};
      reviews
        .filter(coach_id.eq(auth.0.id).and(review_id.eq(id)))
        .first::<Review>(conn)
    })
    .await?
    .into();

  Response::success(review)
}
