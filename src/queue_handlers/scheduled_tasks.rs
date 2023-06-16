use crate::config::Config;
use crate::fairings::sqs::{QueueHandler, QueueHandlerError};
use crate::guards::DbConn;
use anyhow::anyhow;
use serde::Deserialize;

#[derive(Deserialize)]
struct SqsMessageBody {
  #[serde(rename = "recordingId")]
  _recording_id: uuid::Uuid,
  #[serde(rename = "instanceId")]
  instance_id: Option<String>,
}

pub struct ScheduledTasksHandler {
  config: Config,
}

#[async_trait]
impl QueueHandler for ScheduledTasksHandler {
  fn new(_db_conn: DbConn, config: Config) -> Self {
    Self { config }
  }

  fn url(&self) -> String {
    self.config.scheduled_tasks_queue.as_ref().unwrap().clone()
  }

  async fn instance_id(
    &self,
    message: &rusoto_sqs::Message,
    _profile: &rocket::figment::Profile,
  ) -> Result<Option<String>, QueueHandlerError> {
    let message_body = self.parse_message(message)?;
    Ok(message_body.instance_id)
  }

  async fn handle(
    &self,
    _message: &rusoto_sqs::Message,
    _profile: &rocket::figment::Profile,
  ) -> Result<(), QueueHandlerError> {
    Ok(())
  }
}

impl ScheduledTasksHandler {
  fn parse_message(
    &self,
    message: &rusoto_sqs::Message,
  ) -> Result<SqsMessageBody, QueueHandlerError> {
    let body = message
      .body
      .clone()
      .ok_or_else(|| anyhow!("No body found in sqs message"))?;

    let message_body: SqsMessageBody = serde_json::from_str(&body).map_err(anyhow::Error::from)?;

    Ok(message_body)
  }
}
