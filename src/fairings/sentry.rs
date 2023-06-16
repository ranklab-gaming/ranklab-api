use rocket::fairing::{self, Fairing, Info, Kind};
use rocket::{Build, Rocket};
use sentry::ClientInitGuard;
use std::sync::Mutex;

pub struct SentryFairing {
  guard: Mutex<Option<ClientInitGuard>>,
}

impl SentryFairing {
  pub fn fairing() -> impl Fairing {
    Self {
      guard: Mutex::new(None),
    }
  }

  fn init(&self, dsn: Option<String>) {
    match &dsn {
      None => {}
      _ => {
        let guard = sentry::init(dsn);
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
    let sentry_dsn: Option<String> = rocket.figment().extract_inner("sentry_dsn").ok();
    self.init(sentry_dsn);
    Ok(rocket)
  }
}
