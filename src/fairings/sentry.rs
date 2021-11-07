use std::sync::Mutex;

use rocket::fairing::{self, Fairing, Info, Kind};
use rocket::{Build, Rocket};
use sentry::ClientInitGuard;

pub struct SentryFairing {
    guard: Mutex<Option<ClientInitGuard>>,
}

impl SentryFairing {
    pub fn fairing() -> impl Fairing {
        SentryFairing {
            guard: Mutex::new(None),
        }
    }

    fn init(&self, dsn: &str) {
        let guard = sentry::init(dsn);

        if guard.is_enabled() {
            let mut self_guard = self.guard.lock().unwrap();
            *self_guard = Some(guard);
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
        self.init("https://c7b459471051450abcfb5b4e25fa2b2c@o1059892.ingest.sentry.io/6048906");
        Ok(rocket)
    }
}
