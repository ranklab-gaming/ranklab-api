#[macro_use]
extern crate rocket;

#[macro_use]
extern crate diesel;

#[macro_use]
extern crate diesel_migrations;

pub mod config;
pub mod db;
pub mod guards;
pub mod models;
pub mod routes;
pub mod schema;
