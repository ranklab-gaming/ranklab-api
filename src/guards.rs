pub mod auth;
mod auth0_management;
mod db_conn;
mod stripe;
pub use self::stripe::Stripe;
pub use auth::Auth;
pub use auth0_management::Auth0Management;
pub use db_conn::DbConn;

#[macro_export]
macro_rules! try_result {
  ($expr:expr $(,)?) => {
    match $expr {
      Result::Ok(val) => val,
      Result::Err(e) => return rocket::outcome::Outcome::Failure(::std::convert::From::from(e)),
    }
  };
}
