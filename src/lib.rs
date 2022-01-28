#[macro_use]
extern crate rocket;

#[macro_use]
extern crate diesel;

pub mod aws;
pub mod config;
pub mod emails;
pub mod fairings;
pub mod games;
pub mod guards;
pub mod models;
pub mod response;
pub mod routes;
pub mod schema;
