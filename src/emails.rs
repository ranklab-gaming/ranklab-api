use crate::aws;
use crate::config::Config;
use rocket::tokio;
use rusoto_core::HttpClient;
use rusoto_core::Region;
use rusoto_sesv2::{
  BulkEmailContent, BulkEmailEntry, Destination, ReplacementEmailContent, ReplacementTemplate,
  SendBulkEmailRequest, SesV2, SesV2Client, Template,
};

pub struct Recipient {
  email: String,
  template_data: serde_json::Value,
}

pub struct Email {
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
    let aws_access_key_id = config.aws_access_key_id.clone();
    let aws_secret_key = config.aws_secret_key.clone();

    let client = SesV2Client::new_with(
      HttpClient::new().unwrap(),
      aws::CredentialsProvider::new(aws_access_key_id, aws_secret_key),
      Region::EuWest2,
    );

    Email {
      client,
      template_data,
      template_name,
      recipients,
    }
  }

  pub fn deliver(self) {
    if self.recipients.len() == 0 {
      return;
    }

    tokio::spawn(async move {
      let email_request = SendBulkEmailRequest {
        from_email_address: Some("noreply@ranklab.gg".to_owned()),
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

      self.client.send_bulk_email(email_request).await.unwrap();
    });
  }
}
