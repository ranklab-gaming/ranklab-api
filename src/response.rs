use rocket::http::Status;
use rocket::response::status::Custom;
use rocket::response::Responder;
use rocket::serde::json::Json;
use rocket::{response, Request};
use serde::Serialize;
use validator::ValidationErrors;

pub enum Response<T: Serialize> {
    Success(T),
    Status(Status),
    ValidationErrors(ValidationErrors),
}

impl<'r, T: Serialize> Responder<'r, 'static> for Response<T> {
    fn respond_to(self, req: &'r Request<'_>) -> response::Result<'static> {
        match self {
            Response::Success(json) => Custom(Status::Ok, Json(json)).respond_to(req),
            Response::Status(status) => status.respond_to(req),
            Response::ValidationErrors(errors) => {
                Custom(Status::UnprocessableEntity, Json(errors)).respond_to(req)
            }
        }
    }
}
