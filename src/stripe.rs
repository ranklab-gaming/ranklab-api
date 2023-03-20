mod tax;
pub use tax::calculations::{TaxCalculation, TaxCalculationLineItem};
pub use tax::transactions::TaxTransaction;

#[derive(thiserror::Error, Debug)]
pub enum RequestError {
  #[error("Bad request")]
  BadRequest(reqwest::Error),
  #[error(transparent)]
  ServerError(#[from] reqwest::Error),
}
