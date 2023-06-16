use crate::aws::CredentialsProvider;
use crate::config::Config;
use rocket::request::{FromRequest, Outcome};
use rocket::{Request, State};
use rocket_okapi::gen::OpenApiGenerator;
use rocket_okapi::request::{OpenApiFromRequest, RequestHeaderInput};
use rusoto_core::HttpClient;
use rusoto_s3::S3Client;
use rusoto_signature::Region;

pub struct S3(pub S3Client);

#[async_trait]
impl<'r> FromRequest<'r> for S3 {
  type Error = ();

  async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
    let config = request.guard::<&State<Config>>().await.unwrap();

    let client = S3Client::new_with(
      HttpClient::from_builder(hyper::Client::builder(), hyper_tls::HttpsConnector::new()),
      CredentialsProvider::new(
        config.aws_access_key_id.clone(),
        config.aws_secret_key.clone(),
      ),
      Region::EuWest2,
    );

    Outcome::Success(S3(client))
  }
}

impl<'a> OpenApiFromRequest<'a> for S3 {
  fn from_request_input(
    _gen: &mut OpenApiGenerator,
    _name: String,
    _required: bool,
  ) -> rocket_okapi::Result<RequestHeaderInput> {
    Ok(RequestHeaderInput::None)
  }
}

impl S3 {
  pub fn into_inner(self) -> S3Client {
    self.0
  }
}
