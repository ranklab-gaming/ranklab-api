use lazy_static::lazy_static;
use rocket::{figment::Profile, Config};
use std::env;

#[macro_use]
extern crate rocket;

pub mod auth;
pub mod aws;
pub mod config;
pub mod data_types;
pub mod emails;
pub mod fairings;
pub mod games;
pub mod guards;
pub mod intercom;
pub mod models;
pub mod oidc;
pub mod pagination;
pub mod queue_handlers;
pub mod response;
pub mod routes;
pub mod schema;
pub mod views;

pub const TEST_PROFILE: Profile = Profile::const_new("test");
pub const DEBUG_PROFILE: Profile = Config::DEBUG_PROFILE;
pub const RELEASE_PROFILE: Profile = Config::RELEASE_PROFILE;

lazy_static! {
  pub static ref PROFILE: Profile = match env::var("ROCKET_PROFILE") {
    Ok(profile) => Profile::new(&profile),
    Err(_) => Config::DEFAULT_PROFILE,
  };
}
