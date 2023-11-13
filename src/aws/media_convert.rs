use rusoto_core::{Client, HttpClient, RusotoError};
use rusoto_mediaconvert::{DescribeEndpointsError, DescribeEndpointsRequest};
use rusoto_signature::SignedRequest;
use serde::Deserialize;
use serde_json::from_slice;

use crate::config::Config;

#[derive(Deserialize)]
pub struct DescribeEndpointsResponseEndpoint {
  pub url: String,
}

#[derive(Deserialize)]
pub struct DescribeEndpointsResponse {
  pub endpoints: Vec<DescribeEndpointsResponseEndpoint>,
}

pub async fn describe_endpoints(
  config: Config,
) -> Result<DescribeEndpointsResponse, RusotoError<DescribeEndpointsError>> {
  let input: DescribeEndpointsRequest = Default::default();

  let client = Client::new_with(
    super::CredentialsProvider::new(
      config.aws_access_key_id.clone(),
      config.aws_secret_key.clone(),
    ),
    HttpClient::from_builder(hyper::Client::builder(), hyper_tls::HttpsConnector::new()),
  );

  let request_uri = "/2017-08-29/endpoints";
  let encoded = Some(serde_json::to_vec(&input).unwrap());

  let mut request = SignedRequest::new(
    "POST",
    "mediaconvert",
    &rusoto_signature::Region::EuWest2,
    &request_uri,
  );

  request.set_content_type("application/x-amz-json-1.1".to_owned());

  request.set_payload(encoded);

  let mut response = client
    .sign_and_dispatch(request)
    .await
    .map_err(RusotoError::from)?;

  if response.status.as_u16() == 200 {
    let response = response.buffer().await.map_err(RusotoError::HttpDispatch)?;
    let result: DescribeEndpointsResponse =
      from_slice(&response.body).map_err(RusotoError::from)?;
    Ok(result)
  } else {
    let response = response.buffer().await.map_err(RusotoError::HttpDispatch)?;
    Err(DescribeEndpointsError::from_response(response))
  }
}
