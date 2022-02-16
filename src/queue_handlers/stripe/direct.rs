use super::StripeEventHandler;
use crate::{config::Config, guards::DbConn};

pub struct Direct {
  config: Config,
}

impl Direct {}

#[async_trait]
impl StripeEventHandler for Direct {
  fn new(_db_conn: DbConn, config: Config) -> Self {
    Self { config }
  }

  fn url(&self) -> String {
    self.config.stripe_direct_webhooks_queue.clone()
  }

  fn secret(&self) -> String {
    self.config.stripe_direct_webhooks_secret.clone()
  }

  async fn handle_event(
    &self,
    webhook: stripe::WebhookEvent,
    _profile: &rocket::figment::Profile,
  ) -> anyhow::Result<()> {
    match webhook.event_type {
      _ => (),
    }

    //   let recording_id = review.recording_id.clone();

    // let recording = db_conn
    //   .run(move |conn| {
    //     use crate::schema::recordings::dsl::{id, recordings};
    //     recordings
    //       .filter(id.eq(recording_id))
    //       .first::<Recording>(conn)
    //   })
    //   .await?;

    // if recording.stripe_payment_intent_id.is_none() {
    //   return Response::mutation_error(Status::BadRequest);
    // }

    // db_conn
    //   .run(move |conn| {
    //     use crate::schema::recordings::dsl::*;
    //     diesel::update(crate::schema::recordings::table.find(recording_id))
    //       .set((stripe_payment_intent_id.eq::<Option<String>>(None),))
    //       .execute(conn)
    //       .unwrap();
    //   })
    //   .await;

    // let review: ReviewView = db_conn
    //   .run(move |conn| {
    //     use crate::schema::reviews::dsl::*;

    //     diesel::insert_into(reviews)
    //       .values((
    //         recording_id.eq(review.recording_id.clone()),
    //         title.eq(review.title.clone()),
    //         game_id.eq(review.game_id.clone()),
    //         player_id.eq(auth.0.id.clone()),
    //         skill_level.eq(game.unwrap().skill_level as i16),
    //         notes.eq(review.notes.clone()),
    //         stripe_payment_intent_id.eq(recording.stripe_payment_intent_id.unwrap()),
    //       ))
    //       .get_result::<Review>(conn)
    //       .unwrap()
    //   })
    //   .await
    //   .into();

    // let coaches = db_conn
    //   .run(move |conn| {
    //     use crate::schema::coaches::dsl::*;
    //     coaches.load::<Coach>(conn).unwrap()
    //   })
    //   .await;

    // let email = Email::new(
    //   config,
    //   "notification".to_owned(),
    //   json!({
    //       "subject": "New VODs are available",
    //       "title": "There are new VODs available for review!",
    //       "body": "Go to your dashboard to start analyzing them.",
    //       "cta" : "View Available VODs",
    //       "cta_url" : "https://ranklab.gg/dashboard"
    //   }),
    //   coaches
    //     .iter()
    //     .map(|coach| {
    //       Recipient::new(
    //         coach.email.clone(),
    //         json!({
    //           "name": coach.name.clone(),
    //         }),
    //       )
    //     })
    //     .collect(),
    // );

    // email.deliver();

    Ok(())
  }
}
