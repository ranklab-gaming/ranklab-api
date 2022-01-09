#![feature(try_trait_v2)]

#[macro_use]
extern crate rocket;

#[macro_use]
extern crate diesel;

#[macro_use]
extern crate diesel_migrations;

pub mod aws;
pub mod config;
pub mod db;
pub mod fairings;
pub mod games;
pub mod guards;
pub mod models;
pub mod response;
pub mod routes;
pub mod schema;
