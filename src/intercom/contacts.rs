use super::Request;
use crate::config::Config;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct Contact {
  pub email: String,
  pub custom_attributes: HashMap<String, String>,
}

impl Contact {
  pub fn new(email: String, custom_attributes: HashMap<String, String>) -> Self {
    Contact {
      email,
      custom_attributes,
    }
  }

  pub async fn create(&self, config: &Config) -> Result<Contact, reqwest::Error> {
    let client = reqwest::Client::new();
    let request = client.post("https://api.intercom.io/contacts");

    let response = Request::with_headers(request, config)
      .json(self)
      .send()
      .await?;

    let contact = response.json().await?;

    Ok(contact)
  }
}
