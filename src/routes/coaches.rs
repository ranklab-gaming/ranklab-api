use crate::db::DbConn;
use crate::guards::Auth;
use crate::models::{Coach, Game, User};
use crate::response::Response;
use diesel::prelude::*;
use rocket::serde::json::serde_json::to_string;
use rocket::serde::json::Json;
use rocket::Route;
use serde::Deserialize;
use validator::Validate;

#[derive(Deserialize, Validate)]
struct CreateCoachRequest {
    #[validate(email)]
    email: String,
    #[validate(length(min = 1))]
    name: String,
    #[validate(length(min = 1))]
    bio: String,
    game: Game,
}

#[post("/", data = "<coach>")]
async fn create_coach(
    coach: Json<CreateCoachRequest>,
    auth: Auth<User>,
    db_conn: DbConn,
) -> Response<Coach> {
    if let Err(errors) = coach.validate() {
        return Response::ValidationErrors(errors);
    }

    let coach = db_conn
        .run(move |conn| {
            use crate::schema::coaches::dsl::*;

            diesel::insert_into(coaches)
                .values((
                    email.eq(coach.email.clone()),
                    name.eq(coach.name.clone()),
                    bio.eq(coach.bio.clone()),
                    game.eq(to_string(&coach.game).unwrap()),
                    user_id.eq(auth.0.id.clone()),
                ))
                .get_result(conn)
                .unwrap()
        })
        .await;

    Response::Success(coach)
}

pub fn build() -> Vec<Route> {
    routes![create_coach]
}
