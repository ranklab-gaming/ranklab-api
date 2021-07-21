pub mod auth;
pub use auth::Auth;

#[macro_export]
macro_rules! try_result {
    ($expr:expr $(,)?) => {
        match $expr {
            Result::Ok(val) => val,
            Result::Err(e) => {
                return rocket::outcome::Outcome::Failure(::std::convert::From::from(e))
            }
        }
    };
}
