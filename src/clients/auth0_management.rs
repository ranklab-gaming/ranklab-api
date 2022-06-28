use crate::config::Config;
use serde::Serialize;

pub struct Auth0ManagementClient {
  client_id: String,
  client_secret: String,
  base_url: String,
}

#[derive(Serialize)]
struct UpdateUserRequest<'a> {
  email: &'a str,
  client_id: &'a str,
  client_secret: &'a str,
}

impl Auth0ManagementClient {
  pub fn new(config: &Config) -> Self {
    Self {
      client_id: config.auth0_client_id.clone(),
      client_secret: config.auth0_client_secret.clone(),
      base_url: config.auth0_issuer_base_url.clone(),
    }
  }

  pub async fn update_user(&self, auth0_id: String, email: &str) -> anyhow::Result<()> {
    let body = UpdateUserRequest {
      email,
      client_id: &self.client_id,
      client_secret: &self.client_secret,
    };

    let client = reqwest::Client::new();
    client
      .post(format!("{}/api/v2/users/{}", self.base_url, auth0_id).as_str())
      .json(&body)
      .send()
      .await?;

    Ok(())
  }
}
