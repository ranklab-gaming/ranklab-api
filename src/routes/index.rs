use crate::response::{QueryResponse, Response, StatusResponse};
use rocket::http::Status;
use rocket_okapi::openapi;

#[openapi(tag = "Ranklab")]
#[get("/")]
pub async fn get() -> QueryResponse<StatusResponse> {
  Response::status(Status::Ok)
}
