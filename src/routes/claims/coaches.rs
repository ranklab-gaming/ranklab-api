use crate::data_types::UserGame;
use crate::guards::auth::Claims;
use crate::guards::{Auth, DbConn, Stripe};
use crate::models::{Coach, CoachChangeset};
use crate::response::{MutationResponse, Response};
use crate::schema::coaches;
use crate::views::CoachView;
use diesel::prelude::*;
use rocket::serde::json::Json;
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Deserialize)]
struct CountrySpec {
  supported_transfer_countries: Vec<String>,
}

#[derive(Serialize)]
struct CountrySpecParams {}

#[derive(Deserialize, Validate, JsonSchema)]
pub struct CreateCoachRequest {
  #[validate(length(min = 1))]
  name: String,
  #[validate(length(min = 1))]
  bio: String,
  games: Vec<UserGame>,
  country: String,
}

#[openapi(tag = "Ranklab")]
#[post("/claims/coaches", data = "<coach>")]
pub async fn create(
  coach: Json<CreateCoachRequest>,
  auth: Auth<Claims>,
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
            .email(auth.0.email.clone())
            .name(coach.name.clone())
            .bio(coach.bio.clone())
            .games(coach.games.clone().into_iter().map(|g| Some(g)).collect())
            .auth0_id(auth.0.sub.clone())
            .country(coach.country.clone()),
        )
        .get_result(conn)
        .unwrap()
    })
    .await;

  let mut params = stripe::CreateAccount::new();
  params.type_ = Some(stripe::AccountType::Express);
  params.country = Some(&coach.country);

  params.capabilities = Some(
    stripe::CreateAccountCapabilities {
      transfers: Some(
        stripe::CreateAccountCapabilitiesTransfers {
          requested: Some(true.into()),
        }
        .into(),
      ),
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
    }
    .into(),
  );

  let service_agreement = match coach.country.as_str() {
    "US" => "full",
    _ => "recipient",
  };

  params.tos_acceptance = Some(
    stripe::AcceptTos {
      date: None,
      ip: None,
      user_agent: None,
      service_agreement: Some(service_agreement.to_owned().into()),
    }
    .into(),
  );

  params.business_profile = Some(stripe::BusinessProfile {
    mcc: None,
    name: None,
    support_address: None,
    support_email: None,
    support_phone: None,
    support_url: None,
    url: None,
    product_description: Some("Ranklab Coach".to_owned().into()),
  });

  params.settings = Some(
    stripe::AccountSettingsParams {
      treasury: None,
      branding: None,
      card_payments: None,
      payments: None,
      card_issuing: None,
      payouts: None,
    }
    .into(),
  );

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

  Response::success(coach)
}

#[openapi(tag = "Ranklab")]
#[post("/claims/coaches/available_countries")]
pub async fn available_countries(
  _auth: Auth<Claims>,
  stripe: Stripe,
) -> MutationResponse<Vec<String>> {
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
