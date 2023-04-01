use super::{build_request, RequestError};
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

  pub async fn create(&self, config: &Config) -> Result<Contact, RequestError> {
    let client = reqwest::Client::new();
    let request = client.post("https://api.intercom.io/contacts");
    let response = build_request(request, config).json(self).send().await?;

    let contact = match response.error_for_status() {
      Ok(response) => response.json::<Contact>().await.unwrap(),
      Err(err) => {
        if err.status() == Some(reqwest::StatusCode::CONFLICT) {
          return Err(RequestError::Conflict(err));
        }

        return Err(err.into());
      }
    };

    Ok(contact)
  }
}
