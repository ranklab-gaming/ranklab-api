#![feature(box_patterns)]

#[macro_use]
extern crate rocket;

#[macro_use]
extern crate diesel;

#[macro_use]
extern crate stripe as async_stripe;

pub mod aws;
pub mod clients;
pub mod config;
pub mod data_types;
pub mod emails;
pub mod fairings;
pub mod games;
pub mod guards;
pub mod models;
pub mod queue_handlers;
pub mod response;
pub mod routes;
pub mod schema;
pub mod stripe;
pub mod views;
