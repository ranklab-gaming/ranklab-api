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

pub const TEST_PROFILE: rocket::figment::Profile = rocket::figment::Profile::const_new("test");
