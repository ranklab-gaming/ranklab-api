pub mod auth;
mod db_conn;
mod s3;
pub use auth::{Auth, Jwt};
pub use db_conn::DbConn;
pub use s3::S3;

#[macro_export]
macro_rules! try_result {
  ($expr:expr $(,)?) => {
    match $expr {
      Result::Ok(val) => val,
      Result::Err(e) => return rocket::outcome::Outcome::Failure(::std::convert::From::from(e)),
    }
  };
}
