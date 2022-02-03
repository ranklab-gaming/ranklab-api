use crate::clients::StripeClient;
use crate::config::Config;
use rocket::request::{FromRequest, Outcome, Request};
use rocket::State;
use rocket_okapi::{
  gen::OpenApiGenerator,
  request::{OpenApiFromRequest, RequestHeaderInput},
};

pub struct Stripe(pub crate::clients::StripeClient);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Stripe {
  type Error = ();

  async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
    let config = req.guard::<&State<Config>>().await;
    Outcome::Success(Stripe(StripeClient::new(config.as_ref().unwrap())))
  }
}

impl<'a> OpenApiFromRequest<'a> for Stripe {
  fn from_request_input(
    _gen: &mut OpenApiGenerator,
    _name: String,
    _required: bool,
  ) -> rocket_okapi::Result<RequestHeaderInput> {
    Ok(RequestHeaderInput::None)
  }
}
