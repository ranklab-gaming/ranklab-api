use crate::guards::{Auth, DbConn, Jwt};
use crate::models::{Coach, Player};
use crate::pagination::{Paginate, PaginatedResult};
use crate::response::{QueryResponse, Response};
use crate::views::CoachView;
use rocket_okapi::openapi;
use schemars::JsonSchema;

#[derive(FromForm, JsonSchema)]
pub struct ListCoachesQuery {
  page: Option<i64>,
  q: Option<String>,
}

#[openapi(tag = "Ranklab")]
#[get("/player/coaches?<params..>")]
pub async fn list(
  params: ListCoachesQuery,
  _auth: Auth<Jwt<Player>>,
  db_conn: DbConn,
) -> QueryResponse<PaginatedResult<CoachView>> {
  let paginated_coaches: PaginatedResult<Coach> = db_conn
    .run(move |conn| {
      Coach::find_by_query(&params.q.unwrap_or_default())
        .paginate(params.page.unwrap_or(1))
        .load_and_count_pages::<Coach>(conn)
        .unwrap()
    })
    .await;

  let coach_views = paginated_coaches
    .records
    .clone()
    .into_iter()
    .map(Into::into)
    .collect();

  Response::success(paginated_coaches.records(coach_views))
}
