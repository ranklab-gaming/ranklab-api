use crate::guards::{Auth, DbConn, Stripe};
use crate::models::{Coach, CoachChangeset, CoachInvitation, CoachInvitationChangeset};
use crate::response::{MutationResponse, QueryResponse, Response};
use crate::schema::coaches;
use crate::views::CoachView;
use bcrypt::{hash, DEFAULT_COST};
use chrono::Utc;
use diesel::prelude::*;
use rocket::serde::json::Json;
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde::{self, Deserialize, Serialize};
use validator::Validate;

#[derive(Deserialize)]
struct CountrySpec {
  supported_transfer_countries: Vec<String>,
}

#[derive(Serialize)]
struct CountrySpecParams {}

#[derive(Deserialize, Validate, JsonSchema)]
pub struct CreateCoachRequest {
  #[validate(email)]
  email: String,
  #[validate(length(min = 8))]
  password: String,
  #[validate(length(min = 2))]
  name: String,
  #[validate(length(min = 30))]
  bio: String,
  #[validate(length(min = 1), custom = "crate::games::validate_ids")]
  game_ids: Vec<String>,
  #[validate(length(min = 1))]
  country: String,
}

#[derive(Deserialize, JsonSchema, Validate)]
#[schemars(rename = "CoachUpdateAccountRequest")]
pub struct UpdateAccountRequest {
  #[validate(length(min = 2))]
  name: String,
  #[validate(email)]
  email: String,
  #[validate(length(min = 1), custom = "crate::games::validate_ids")]
  game_ids: Vec<String>,
  #[validate(length(min = 1))]
  bio: String,
}

#[openapi(tag = "Ranklab")]
#[get("/coach/account")]
pub async fn get(auth: Auth<Coach>) -> QueryResponse<CoachView> {
  Response::success(auth.0.into())
}

#[openapi(tag = "Ranklab")]
#[put("/coach/account", data = "<account>")]
pub async fn update(
  account: Json<UpdateAccountRequest>,
  auth: Auth<Coach>,
  db_conn: DbConn,
) -> MutationResponse<CoachView> {
  if let Err(errors) = account.validate() {
    return Response::validation_error(errors);
  }

  let coach = auth.0.clone();

  let coach: CoachView = db_conn
    .run(move |conn| {
      diesel::update(&coach)
        .set(
          CoachChangeset::default()
            .email(account.email.clone())
            .name(account.name.clone())
            .bio(account.bio.clone())
            .game_ids(account.game_ids.clone().into_iter().map(Some).collect()),
        )
        .get_result::<Coach>(conn)
        .unwrap()
    })
    .await
    .into();

  Response::success(coach)
}

#[openapi(tag = "Ranklab")]
#[post("/coach/account", data = "<coach>")]
pub async fn create(
  auth: Auth<CoachInvitation>,
  coach: Json<CreateCoachRequest>,
  db_conn: DbConn,
  stripe: Stripe,
) -> MutationResponse<CoachView> {
  if let Err(errors) = coach.validate() {
    return Response::validation_error(errors);
  }

  let coach: Coach = db_conn
    .run(move |conn| {
      diesel::insert_into(coaches::table)
        .values(
          CoachChangeset::default()
            .email(coach.email.clone())
            .password(hash(coach.password.clone(), DEFAULT_COST).expect("Failed to hash password"))
            .name(coach.name.clone())
            .bio(coach.bio.clone())
            .game_ids(coach.game_ids.clone().into_iter().map(Some).collect())
            .country(coach.country.clone()),
        )
        .get_result(conn)
        .unwrap()
    })
    .await;

  let mut params = stripe::CreateAccount::new();
  params.type_ = Some(stripe::AccountType::Express);
  params.country = Some(&coach.country);

  params.capabilities = Some(stripe::CreateAccountCapabilities {
    transfers: Some(stripe::CreateAccountCapabilitiesTransfers {
      requested: Some(true),
    }),
    affirm_payments: None,
    bank_transfer_payments: None,
    link_payments: None,
    paynow_payments: None,
    treasury: None,
    us_bank_account_ach_payments: None,
    acss_debit_payments: None,
    afterpay_clearpay_payments: None,
    au_becs_debit_payments: None,
    bacs_debit_payments: None,
    bancontact_payments: None,
    boleto_payments: None,
    card_issuing: None,
    card_payments: None,
    cartes_bancaires_payments: None,
    eps_payments: None,
    fpx_payments: None,
    giropay_payments: None,
    grabpay_payments: None,
    ideal_payments: None,
    jcb_payments: None,
    klarna_payments: None,
    legacy_payments: None,
    oxxo_payments: None,
    p24_payments: None,
    sepa_debit_payments: None,
    sofort_payments: None,
    tax_reporting_us_1099_k: None,
    tax_reporting_us_1099_misc: None,
    konbini_payments: None,
    blik_payments: None,
    promptpay_payments: None,
    india_international_payments: None,
  });

  let service_agreement = match coach.country.as_str() {
    "US" => "full",
    _ => "recipient",
  };

  params.tos_acceptance = Some(stripe::AcceptTos {
    date: None,
    ip: None,
    user_agent: None,
    service_agreement: Some(service_agreement.to_owned()),
  });

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
    treasury: None,
    branding: None,
    card_payments: None,
    payments: None,
    card_issuing: None,
    payouts: None,
  });

  let account = stripe::Account::create(&stripe.0 .0, params).await.unwrap();

  let coach: CoachView = db_conn
    .run(move |conn| {
      diesel::update(&coach)
        .set(CoachChangeset::default().stripe_account_id(Some(account.id.to_string())))
        .get_result::<Coach>(conn)
        .unwrap()
    })
    .await
    .into();

  db_conn
    .run(move |conn| {
      diesel::update(&auth.0)
        .set(CoachInvitationChangeset::default().used_at(Some(Utc::now().naive_utc())))
        .get_result::<CoachInvitation>(conn)
        .unwrap()
    })
    .await;

  Response::success(coach)
}

#[openapi(tag = "Ranklab")]
#[post("/coach/countries")]
pub async fn get_countries(stripe: Stripe) -> MutationResponse<Vec<String>> {
  let country_spec = &stripe
    .0
     .0
    .get_query::<CountrySpec, CountrySpecParams>(
      &format!("/country_specs/{}", "US"),
      CountrySpecParams {},
    )
    .await
    .unwrap();

  Response::success(country_spec.supported_transfer_countries.clone())
}
