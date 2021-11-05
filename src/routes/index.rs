use rocket::serde::json::Json;
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde::Serialize;

#[derive(Serialize, JsonSchema)]
pub struct Health {
    status: String,
}

#[openapi(tag = "Ranklab")]
#[get("/")]
pub async fn get_health() -> Json<Health> {
    Json(Health {
        status: "ok".into(),
    })
}
