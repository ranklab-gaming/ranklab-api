use diesel_derive_enum::DbEnum;

#[derive(DbEnum, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum ReviewState {
  AwaitingPayment,
  AwaitingReview,
  Draft,
  Published,
  Accepted,
  Refunded,
}
