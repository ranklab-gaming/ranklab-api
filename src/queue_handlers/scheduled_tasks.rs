use crate::clients::StripeClient;
use crate::config::Config;
use crate::data_types::ReviewState;
use crate::emails::{Email, Recipient};
use crate::fairings::sqs::{QueueHandler, QueueHandlerError};
use crate::guards::DbConn;
use crate::models::{Coach, Player, Review};
use crate::schema::{coaches, players, reviews};
use anyhow::anyhow;
use diesel::prelude::*;
use serde::Deserialize;
use serde_json::json;
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
      let client = self
        .client
        .as_ref()
        .clone()
        .with_strategy(stripe::RequestStrategy::Idempotent(review.id.to_string()));

      let payment_intent = review.get_payment_intent(&client).await;
      let mut create_refund = CreateRefund::new();

      create_refund.payment_intent = Some(payment_intent.id.clone());

      stripe::Refund::create(&client, create_refund)
        .await
        .map_err(anyhow::Error::from)?;

      let player_id = review.player_id;
      let coach_id = review.coach_id;

      let player = self
        .db_conn
        .run(move |conn| players::table.find(&player_id).get_result::<Player>(conn))
        .await?;

      let coach = self
        .db_conn
        .run(move |conn| coaches::table.find(&coach_id).get_result::<Coach>(conn))
        .await?;

      let email = Email::new(
        &self.config,
        "notification".to_owned(),
        json!({
          "subject": "Your review has been refunded",
          "title": "Your review has been refunded.",
          "body": format!("{} hasn't reviewed your recording within the 5-days window, and your review has been automatically refunded. You can now find the review in your archive.", coach.name),
          "cta" : "View Archive",
          "cta_url" : format!("{}/player/archive", self.config.web_host),
          "unsubscribe_url": format!("{}/player/account?tab=notifications", self.config.web_host),
        }),
        vec![Recipient::new(
          player.email.clone(),
          json!({
            "name": player.name,
          }),
        )],
      );

      email.deliver().await.map_err(anyhow::Error::from)?;
    }

    Ok(())
  }
}
