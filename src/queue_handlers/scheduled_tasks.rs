use crate::clients::StripeClient;
use crate::config::Config;
use crate::data_types::ReviewState;
use crate::fairings::sqs::{QueueHandler, QueueHandlerError};
use crate::guards::DbConn;
use crate::models::Review;
use crate::schema::reviews;
use anyhow::anyhow;
use diesel::prelude::*;
use serde::Deserialize;
use stripe::CreateRefund;

#[derive(Deserialize)]
struct SqsMessageBody {
  #[serde(rename = "reviewId")]
  review_id: uuid::Uuid,
}

pub struct ScheduledTasksHandler {
  db_conn: DbConn,
  config: Config,
  client: StripeClient,
}

#[async_trait]
impl QueueHandler for ScheduledTasksHandler {
  fn new(db_conn: DbConn, config: Config) -> Self {
    let client = StripeClient::new(&config);

    Self {
      db_conn,
      config,
      client,
    }
  }

  fn url(&self) -> String {
    self.config.scheduled_tasks_queue.as_ref().unwrap().clone()
  }

  async fn handle(
    &self,
    message: &rusoto_sqs::Message,
    _profile: &rocket::figment::Profile,
  ) -> Result<(), QueueHandlerError> {
    let body = message
      .body
      .clone()
      .ok_or_else(|| anyhow!("No body found in sqs message"))?;

    let message_body: SqsMessageBody = serde_json::from_str(&body).map_err(anyhow::Error::from)?;

    let review = self
      .db_conn
      .run(move |conn| {
        reviews::table
          .find(&message_body.review_id)
          .get_result::<Review>(conn)
      })
      .await?;

    if review.state == ReviewState::AwaitingReview {
      let payment_intent = review.get_payment_intent(&self.client.0).await;
      let mut create_refund = CreateRefund::new();

      create_refund.payment_intent = Some(payment_intent.id.clone());

      stripe::Refund::create(&self.client.0, create_refund)
        .await
        .map_err(anyhow::Error::from)?;
    }

    Ok(())
  }
}
