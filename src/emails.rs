use crate::aws::ConfigCredentialsProvider;
use crate::config::Config;
use hyper_tls::HttpsConnector;
use rusoto_core::{HttpClient, Region, RusotoError};
use rusoto_sesv2::{
  BulkEmailContent, BulkEmailEntry, Destination, ReplacementEmailContent, ReplacementTemplate,
  SendBulkEmailError, SendBulkEmailRequest, SesV2, SesV2Client, Template,
};
use serde::Serialize;

#[derive(Serialize)]
pub struct Recipient {
  email: String,
  template_data: serde_json::Value,
}

#[derive(Serialize)]
pub struct Email {
  #[serde(skip)]
  client: SesV2Client,
  template_data: serde_json::Value,
  template_name: String,
  recipients: Vec<Recipient>,
}

impl Recipient {
  pub fn new(email: String, template_data: serde_json::Value) -> Self {
    Recipient {
      email,
      template_data,
    }
  }
}

impl Email {
  pub fn new(
    config: &Config,
    template_name: String,
    template_data: serde_json::Value,
    recipients: Vec<Recipient>,
  ) -> Self {
    let client = SesV2Client::new_with(
      HttpClient::from_connector(HttpsConnector::new()),
      ConfigCredentialsProvider::new(config.clone()),
      Region::EuWest2,
    );

    Email {
      client,
      template_data,
      template_name,
      recipients,
    }
  }

  pub async fn deliver(self) -> Result<(), RusotoError<SendBulkEmailError>> {
    if self.recipients.is_empty() {
      return Ok(());
    }

    let email_request = SendBulkEmailRequest {
      from_email_address: Some("Ranklab <noreply@ranklab.gg>".to_string()),
      default_content: BulkEmailContent {
        template: Some(Template {
          template_name: Some(self.template_name),
          template_data: Some(self.template_data.to_string()),
          ..Default::default()
        }),
      },
      bulk_email_entries: self
        .recipients
        .iter()
        .map(|recipient| BulkEmailEntry {
          destination: Destination {
            to_addresses: Some(vec![recipient.email.clone()]),
            ..Default::default()
          },
          replacement_email_content: Some(ReplacementEmailContent {
            replacement_template: Some(ReplacementTemplate {
              replacement_template_data: Some(recipient.template_data.to_string()),
            }),
          }),
          ..Default::default()
        })
        .collect(),
      ..Default::default()
    };

    self.client.send_bulk_email(email_request).await?;

    Ok(())
  }
}
