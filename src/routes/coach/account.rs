use std::collections::HashMap;

use crate::auth::{generate_token, Account};
use crate::config::Config;
use crate::emails::{Email, Recipient};
use crate::guards::{Auth, DbConn, Jwt, Stripe};
use crate::models::{Avatar, Coach, CoachChangeset, CoachInvitation, CoachInvitationChangeset};
use crate::response::{MutationError, MutationResponse, QueryResponse, Response};
use crate::routes::session::CreateSessionResponse;
use crate::schema::coaches;
use crate::views::CoachView;
use bcrypt::{hash, DEFAULT_COST};
use chrono::Utc;
use diesel::prelude::*;
use diesel::result::DatabaseErrorKind;
use rocket::figment::Provider;
use rocket::serde::json::Json;
use rocket::State;
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde::{self, Deserialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use slugify::slugify;
use validator::{Validate, ValidationError, ValidationErrors};

#[derive(Deserialize, Validate, JsonSchema)]
pub struct CreateCoachRequest {
  #[validate(length(min = 2))]
  name: String,
  #[validate(email)]
  email: String,
  #[validate(length(min = 8))]
  password: String,
  #[validate(length(min = 30))]
  bio: String,
  #[validate(length(min = 1), custom = "crate::games::validate_id")]
  game_id: String,
  #[validate(length(min = 1))]
  country: String,
  #[validate(range(min = 1, max = 10000))]
  price: i32,
}

#[derive(Deserialize, JsonSchema, Validate)]
pub struct UpdateCoachRequest {
  #[validate(length(min = 2))]
  name: String,
  #[validate(email)]
  email: String,
  #[validate(length(min = 1))]
  bio: String,
  #[validate(range(min = 1, max = 10000))]
  price: i32,
  emails_enabled: bool,
}

#[openapi(tag = "Ranklab")]
#[get("/coach/account")]
pub async fn get(
  auth: Auth<Jwt<Coach>>,
  config: &State<Config>,
  db_conn: DbConn,
) -> QueryResponse<CoachView> {
  let coach = auth.into_deep_inner();

  let avatar: Option<Avatar> = match coach.avatar_id {
    Some(avatar_id) => db_conn
      .run(move |conn| Avatar::find_by_id(&avatar_id).get_result::<Avatar>(conn))
      .await
      .ok(),
    None => None,
  };

  Response::success(CoachView::new(coach, Some(config), avatar))
}

#[openapi(tag = "Ranklab")]
#[put("/coach/account", data = "<account>")]
pub async fn update(
  account: Json<UpdateCoachRequest>,
  auth: Auth<Jwt<Coach>>,
  db_conn: DbConn,
  config: &State<Config>,
) -> MutationResponse<CoachView> {
  if let Err(errors) = account.validate() {
    return Response::validation_error(errors);
  }

  let coach = auth.into_deep_inner();

  let coach = db_conn
    .run(move |conn| {
      diesel::update(&coach)
        .set(
          CoachChangeset::default()
            .email(account.email.clone())
            .name(account.name.clone())
            .bio(account.bio.clone())
            .price(account.price)
            .emails_enabled(account.emails_enabled)
            .slug(slugify!(&account.name)),
        )
        .get_result::<Coach>(conn)
    })
    .await
    .map_err(|err| match &err {
      diesel::result::Error::DatabaseError(DatabaseErrorKind::UniqueViolation, info) => {
        if let Some(name) = info.constraint_name() {
          if name == "coaches_email_key" {
            let mut errors = ValidationErrors::new();
            errors.add("email", ValidationError::new("uniqueness"));
            return MutationError::ValidationErrors(errors);
          }

          if name == "coaches_slug_key" {
            let mut errors = ValidationErrors::new();
            errors.add("name", ValidationError::new("uniqueness"));
            return MutationError::ValidationErrors(errors);
          }
        };

        MutationError::InternalServerError(err.into())
      }
      _ => MutationError::InternalServerError(err.into()),
    })?;

  Response::success(CoachView::new(coach, Some(config), None))
}

#[openapi(tag = "Ranklab")]
#[post("/coach/account", data = "<coach>")]
pub async fn create(
  auth: Auth<CoachInvitation>,
  coach: Json<CreateCoachRequest>,
  db_conn: DbConn,
  stripe: Stripe,
  config: &State<Config>,
  rocket_config: &rocket::Config,
) -> MutationResponse<CreateSessionResponse> {
  if let Err(errors) = coach.validate() {
    return Response::validation_error(errors);
  }

  let mut params = stripe::CreateAccount::new();
  let profile = rocket_config.profile().unwrap();

  params.type_ = Some(stripe::AccountType::Express);
  params.country = Some(&coach.country);
  params.email = Some(&coach.email);

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
    tax_reporting_us_1099_misc: match coach.country.as_str() {
      "US" => Some(stripe::CreateAccountCapabilitiesTaxReportingUs1099Misc {
        requested: Some(true),
      }),
      _ => None,
    },
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

  let mut metadata = HashMap::new();

  if let Some(instance_id) = config.instance_id.as_ref() {
    metadata.insert("instance_id".to_owned(), instance_id.to_owned());
  }

  params.metadata = Some(metadata);

  let stripe = stripe
    .into_inner()
    .with_strategy(stripe::RequestStrategy::Idempotent(hex::encode(
      Sha256::digest(coach.email.as_bytes()),
    )));

  let account = stripe::Account::create(&stripe, params).await.unwrap();

  let coach: Coach = db_conn
    .run(move |conn| {
      diesel::insert_into(coaches::table)
        .values(
          CoachChangeset::default()
            .email(coach.email.clone())
            .password(hash(coach.password.clone(), DEFAULT_COST).unwrap())
            .stripe_account_id(account.id.to_string())
            .name(coach.name.clone())
            .bio(coach.bio.clone())
            .price(coach.price)
            .game_id(coach.game_id.clone())
            .country(coach.country.clone())
            .slug(slugify!(&coach.name)),
        )
        .get_result(conn)
    })
    .await
    .map_err(|err| match &err {
      diesel::result::Error::DatabaseError(DatabaseErrorKind::UniqueViolation, info) => {
        if let Some(name) = info.constraint_name() {
          if name == "coaches_email_key" {
            let mut errors = ValidationErrors::new();
            errors.add("email", ValidationError::new("uniqueness"));
            return MutationError::ValidationErrors(errors);
          }

          if name == "coaches_slug_key" {
            let mut errors = ValidationErrors::new();
            errors.add("name", ValidationError::new("uniqueness"));
            return MutationError::ValidationErrors(errors);
          }
        };

        MutationError::InternalServerError(err.into())
      }
      _ => MutationError::InternalServerError(err.into()),
    })?;

  db_conn
    .run(move |conn| {
      diesel::update(&auth.into_inner())
        .set(CoachInvitationChangeset::default().used_at(Some(Utc::now().naive_utc())))
        .get_result::<CoachInvitation>(conn)
        .unwrap()
    })
    .await;

  let slug = coach.slug.clone();
  let name = coach.name.clone();
  let email = coach.email.clone();
  let account = Account::Coach(coach);
  let token = generate_token(&account, config);

  let coach_signup_email = Email::new(
    config,
    "notification".to_owned(),
    json!({
      "subject": "A coach has signed up!",
      "title": format!("{} has signed up to Ranklab", name),
      "body": format!("Their email is: {}", email),
      "cta" : "Go to Profile",
      "cta_url" : format!("{}/r/{}", config.web_host, slug),
    }),
    vec![Recipient::new(
      "sales@ranklab.gg".to_owned(),
      json!({
        "name": "Ranklab",
      }),
    )],
  );

  if profile == rocket::config::Config::RELEASE_PROFILE {
    coach_signup_email.deliver().await.unwrap();
  }

  Response::success(CreateSessionResponse { token })
}
