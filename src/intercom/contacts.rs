use super::Request;
use crate::config::Config;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct Contact {
  pub role: Option<String>,
  pub external_id: Option<String>,
  pub email: String,
  pub phone: Option<String>,
  pub name: Option<String>,
  pub avatar: Option<String>,
  pub signed_up_at: Option<i32>,
  pub last_seen_at: Option<i32>,
  pub owner_id: Option<i32>,
  pub unsubscribed_from_emails: Option<bool>,
  pub custom_attributes: HashMap<String, String>,
}

impl Contact {
  pub fn new(email: String, custom_attributes: HashMap<String, String>) -> Self {
    Contact {
      role: None,
      external_id: None,
      email,
      phone: None,
      name: None,
      avatar: None,
      signed_up_at: None,
      last_seen_at: None,
      owner_id: None,
      unsubscribed_from_emails: None,
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
