use std::sync::Mutex;

use rocket::fairing::{self, Fairing, Info, Kind};
use rocket::{Build, Rocket};
use sentry::ClientInitGuard;

pub struct SentryFairing {
  dsn: String,
  guard: Mutex<Option<ClientInitGuard>>,
}

impl SentryFairing {
  pub fn fairing(dsn: String) -> impl Fairing {
    Self {
      dsn: dsn,
      guard: Mutex::new(None),
    }
  }

  fn init(&self) {
    match &self.dsn.len() {
      0 => {}
      _length => {
        let guard = sentry::init(self.dsn.clone());
        *self.guard.lock().unwrap() = Some(guard);
      }
    }
  }
}

#[rocket::async_trait]
impl Fairing for SentryFairing {
  fn info(&self) -> Info {
    Info {
      name: "sentry",
      kind: Kind::Ignite,
    }
  }

  async fn on_ignite(&self, rocket: Rocket<Build>) -> fairing::Result {
    self.init();
    Ok(rocket)
  }
}
