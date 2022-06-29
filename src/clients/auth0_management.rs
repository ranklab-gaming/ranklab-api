mod error;
mod token;
use crate::config::Config;
use hyper::Method;
use reqwest::RequestBuilder;
use serde::Serialize;

use self::error::{Auth0Error, Auth0ErrorResponse};
use self::token::TokenManager;

pub struct Auth0ManagementClient {
  base_url: String,
  token: TokenManager,
}

#[derive(Serialize)]
struct UpdateUserRequest<'a> {
  email: &'a str,
}

impl Auth0ManagementClient {
  pub fn new(config: &Config) -> Self {
    Self {
      base_url: config.auth0_issuer_base_url.clone(),
      token: TokenManager::new(
        &config.auth0_issuer_base_url,
        &format!("{}api/v2/", config.auth0_issuer_base_url),
        &config.auth0_client_id,
        &config.auth0_client_secret,
      ),
    }
  }

  pub async fn update_user(&self, auth0_id: String, email: &str) -> Result<(), Auth0Error> {
    let body = UpdateUserRequest { email };
    let client = reqwest::Client::new();

    self
      .send(
        client
          .request(
            Method::PATCH,
            &format!("{}api/v2/users/{}", self.base_url, auth0_id),
          )
          .json(&body),
      )
      .await
  }

  pub async fn send(&self, req: RequestBuilder) -> Result<(), Auth0Error> {
    let token = self.token.get_token().await?;
    let res = req //
      .bearer_auth(&token)
      .send()
      .await?;

    if res.status().is_success() {
      Ok(())
    } else {
      let body = res.bytes().await?;
      let body = body.to_vec();
      let body = std::str::from_utf8(&body).unwrap();

      let err = serde_json::from_str::<Auth0ErrorResponse>(body);
      if let Ok(err) = err {
        Err(Auth0Error::from(err))
      } else {
        Err(Auth0Error::Auth0(body.to_owned()))
      }
    }
  }
}
