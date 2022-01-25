use crate::guards::auth::Claims;
use crate::guards::Auth;
use crate::guards::DbConn;
use crate::guards::Stripe;
use crate::models::Coach;
use crate::models::UserGame;
use crate::response::{MutationResponse, Response};
use diesel::prelude::*;
use rocket::serde::json::Json;
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde::Deserialize;
use validator::Validate;

#[derive(Deserialize, Validate, JsonSchema)]
pub struct CreateCoachRequest {
  #[validate(length(min = 1))]
  name: String,
  #[validate(length(min = 1))]
  bio: String,
  games: Vec<UserGame>,
}

#[openapi(tag = "Ranklab")]
#[post("/claims/coaches", data = "<coach>")]
pub async fn create(
  coach: Json<CreateCoachRequest>,
  auth: Auth<Claims>,
  db_conn: DbConn,
  stripe: Stripe,
) -> MutationResponse<Coach> {
  if let Err(errors) = coach.validate() {
    return Response::validation_error(errors);
  }

  let coach: Coach = db_conn
    .run(move |conn| {
      use crate::schema::coaches::dsl::*;

      diesel::insert_into(coaches)
        .values((
          email.eq(auth.0.email.clone()),
          name.eq(coach.name.clone()),
          bio.eq(coach.bio.clone()),
          games.eq(coach.games.clone()),
          auth0_id.eq(auth.0.sub.clone()),
        ))
        .get_result(conn)
        .unwrap()
    })
    .await;

  let mut params = stripe::CreateAccount::new();
  params.type_ = Some(stripe::AccountType::Express);
  params.business_type = Some(stripe::BusinessType::Individual);
  params.requested_capabilities = Some(vec![stripe::RequestedCapability::Transfers]);

  params.business_profile = Some(stripe::BusinessProfile {
    mcc: None,
    name: None,
    support_address: None,
    support_email: None,
    support_phone: None,
    support_url: None,
    url: None,
    product_description: Some("Ranklab Coach".to_owned()),
  });

  params.settings = Some(stripe::AccountSettingsParams {
    branding: None,
    card_payments: None,
    payments: None,
    payouts: Some(stripe::PayoutSettingsParams {
      debit_negative_balances: None,
      statement_descriptor: None,
      schedule: Some(stripe::TransferScheduleParams {
        delay_days: Some(stripe::DelayDays::Days(7)),
        interval: None,
        monthly_anchor: None,
        weekly_anchor: None,
      }),
    }),
  });

  let account = stripe::Account::create(&stripe.0, params).await.unwrap();

  let coach: Coach = db_conn
    .run(move |conn| {
      use crate::schema::coaches::dsl::*;

      diesel::update(crate::schema::coaches::table.find(coach.id))
        .set(stripe_account_id.eq(Some(account.id.to_string())))
        .get_result(conn)
        .unwrap()
    })
    .await;

  Response::success(coach)
}
