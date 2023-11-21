use hyper_tls::HttpsConnector;
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
  let client = Client::new_with(
    super::ConfigCredentialsProvider::new(config.clone()),
    HttpClient::from_connector(HttpsConnector::new()),
  );

  let input: DescribeEndpointsRequest = Default::default();
  let request_uri = "/2017-08-29/endpoints";
  let encoded = Some(serde_json::to_vec(&input).unwrap());

  let mut request = SignedRequest::new(
    "POST",
    "mediaconvert",
    &rusoto_signature::Region::EuWest2,
    request_uri,
  );

  request.set_content_type("application/x-amz-json-1.1".to_owned());
  request.set_payload(encoded);

  let mut response = client
    .sign_and_dispatch(request)
    .await
    .map_err(RusotoError::from)?;

  let response = response.buffer().await.map_err(RusotoError::HttpDispatch)?;

  if response.status.as_u16() == 200 {
    Ok(from_slice(&response.body).map_err(RusotoError::from)?)
  } else {
    Err(DescribeEndpointsError::from_response(response))
  }
}
