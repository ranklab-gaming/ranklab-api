use crate::clients::Auth0ManagementClient;
use crate::config::Config;
use rocket::request::{FromRequest, Outcome, Request};
use rocket::State;
use rocket_okapi::gen::OpenApiGenerator;
use rocket_okapi::request::{OpenApiFromRequest, RequestHeaderInput};

pub struct Auth0Management(pub crate::clients::Auth0ManagementClient);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Auth0Management {
  type Error = ();

  async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
    let config = req.guard::<&State<Config>>().await;
    let client = Auth0ManagementClient::new(config.as_ref().unwrap());
    Outcome::Success(Self(client))
  }
}

impl<'a> OpenApiFromRequest<'a> for Auth0 {
  fn from_request_input(
    _gen: &mut OpenApiGenerator,
    _name: String,
    _required: bool,
  ) -> rocket_okapi::Result<RequestHeaderInput> {
    Ok(RequestHeaderInput::None)
  }
}
