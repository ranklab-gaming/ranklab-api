use crate::clients::StripeClient;
use crate::config::Config;
use rocket::request::{FromRequest, Outcome, Request};
use rocket::State;
use rocket_okapi::gen::OpenApiGenerator;
use rocket_okapi::request::{OpenApiFromRequest, RequestHeaderInput};

pub struct Stripe(StripeClient);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Stripe {
  type Error = ();

  async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
    let config = req.guard::<&State<Config>>().await;
    let client = StripeClient::new(config.as_ref().unwrap());
    Outcome::Success(Stripe(client))
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

impl Stripe {
  pub fn into_inner(self) -> stripe::Client {
    self.0.into_inner()
  }
}
