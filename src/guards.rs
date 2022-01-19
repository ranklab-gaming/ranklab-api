pub mod auth;
pub mod db_conn;
pub mod stripe;
pub use self::stripe::Stripe;
pub use auth::Auth;
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
