use serde_json::json;

use crate::{config::Config, models::Coach};

use super::{Email, Recipient};

pub fn coach_has_reviews(config: &Config, coach: &Coach) -> Email {
  Email::new(
    config,
    "notification".to_owned(),
    json!({
      "subject": "New recordings are waiting for your review",
      "title": "There are new recordings available for review!",
      "body": "Go to your dashboard to start analyzing them.",
      "cta" : "View Available Recordings",
      "cta_url" : format!("{}/coach/dashboard", config.web_host),
      "unsubscribe_url": format!("{}/coach/account?tab=notifications", config.web_host)
    }),
    vec![Recipient::new(
      coach.email.clone(),
      json!({
        "name": coach.name,
      }),
    )],
  )
}
