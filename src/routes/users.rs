use crate::auth::{decode_token_credentials, generate_token, Credentials};
use crate::config::Config;
use crate::emails::{Email, Recipient};
use crate::guards::{Auth, DbConn, Jwt};
use crate::models::{Avatar, Session, User, UserChangeset};
use crate::response::{MutationError, MutationResponse, QueryResponse, Response};
use crate::schema::users;
use crate::views::UserView;
use bcrypt::{hash, DEFAULT_COST};
use diesel::prelude::*;
use diesel::result::DatabaseErrorKind;
use rocket::figment::Provider;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde;
use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;
use uuid::Uuid;
use validator::{Validate, ValidationError, ValidationErrors};

#[derive(Deserialize, JsonSchema, Validate)]
pub struct UpdateUserRequest {
  #[validate(length(min = 2))]
  name: String,
  emails_enabled: bool,
  avatar_id: Option<Uuid>,
}

#[derive(Deserialize, Validate, JsonSchema)]
pub struct CreateUserRequest {
  #[validate(length(min = 2))]
  name: String,
  credentials: Credentials,
}

#[openapi(tag = "Ranklab")]
#[get("/users")]
pub async fn get(
  auth: Auth<Jwt>,
  config: &State<Config>,
  db_conn: DbConn,
) -> QueryResponse<UserView> {
  let user = auth.into_user();

  let avatar: Option<Avatar> = match user.avatar_id {
    Some(avatar_id) => db_conn
      .run(move |conn| Avatar::find_processed_by_id(&avatar_id).get_result::<Avatar>(conn))
      .await
      .ok(),
    None => None,
  };

  Response::success(UserView::new(user, Some(config), avatar))
}

#[openapi(tag = "Ranklab")]
#[post("/users", data = "<user>")]
pub async fn create(
  user: Json<CreateUserRequest>,
  db_conn: DbConn,
  config: &State<Config>,
  rocket_config: &rocket::Config,
) -> MutationResponse<Session> {
  let profile = rocket_config.profile().unwrap();

  if let Err(errors) = user.validate() {
    return Response::validation_error(errors);
  }

  let email = match &user.credentials {
    Credentials::Password(credentials) => credentials.email.clone(),
    Credentials::Token(credentials) => decode_token_credentials(&credentials, config)
      .ok_or_else(|| MutationError::Status(Status::UnprocessableEntity))?
      .sub
      .clone(),
  };

  let password = match &user.credentials {
    Credentials::Password(credentials) => Some(credentials.password.clone()),
    Credentials::Token(_) => None,
  };

  let mut metadata = HashMap::new();

  if let Some(instance_id) = config.instance_id.as_ref() {
    metadata.insert("instance_id".to_owned(), instance_id.to_owned());
  }

  let user = db_conn
    .run(move |conn| {
      diesel::insert_into(users::table)
        .values(
          UserChangeset::default()
            .password(password.map(|password| hash(password.clone(), DEFAULT_COST).unwrap()))
            .email(email.clone())
            .name(user.name.clone()),
        )
        .get_result::<User>(conn)
    })
    .await
    .map_err(|err| match &err {
      diesel::result::Error::DatabaseError(DatabaseErrorKind::UniqueViolation, info) => {
        if let Some(name) = info.constraint_name() {
          if name == "users_email_key" {
            let mut errors = ValidationErrors::new();
            errors.add("email", ValidationError::new("uniqueness"));
            return MutationError::ValidationErrors(errors);
          }
        };

        MutationError::InternalServerError(err.into())
      }
      _ => MutationError::InternalServerError(err.into()),
    })?;

  let name = user.name.clone();
  let email = user.email.clone();
  let token = generate_token(&user, config);

  let user_signup_email = Email::new(
    config,
    "notification".to_owned(),
    json!({
      "subject": "Someone has signed up!",
      "title": format!("{} has signed up to Ranklab", name),
      "body": format!("Their email is: {}", email),
    }),
    vec![Recipient::new(
      "sales@ranklab.gg".to_owned(),
      json!({
        "name": "Ranklab",
      }),
    )],
  );

  if profile == rocket::config::Config::RELEASE_PROFILE {
    user_signup_email.deliver().await.unwrap();
  }

  Response::success(Session { token })
}

#[openapi(tag = "Ranklab")]
#[put("/users", data = "<user>")]
pub async fn update(
  user: Json<UpdateUserRequest>,
  auth: Auth<Jwt>,
  db_conn: DbConn,
  config: &State<Config>,
) -> MutationResponse<UserView> {
  if let Err(errors) = user.validate() {
    return Response::validation_error(errors);
  }

  let existing_user = auth.into_user();
  let avatar_id = user.avatar_id.clone();

  let avatar = match avatar_id {
    Some(avatar_id) => db_conn
      .run(move |conn| Avatar::find_processed_by_id(&avatar_id).get_result::<Avatar>(conn))
      .await
      .ok(),
    None => None,
  };

  let user: User = db_conn
    .run(move |conn| {
      diesel::update(&existing_user)
        .set(
          UserChangeset::default()
            .name(user.name.clone())
            .emails_enabled(user.emails_enabled)
            .avatar_id(avatar_id),
        )
        .get_result::<User>(conn)
    })
    .await
    .map_err(|err| match &err {
      diesel::result::Error::DatabaseError(DatabaseErrorKind::UniqueViolation, info) => {
        if let Some(name) = info.constraint_name() {
          if name == "users_email_key" {
            let mut errors = ValidationErrors::new();
            errors.add("email", ValidationError::new("uniqueness"));
            return MutationError::ValidationErrors(errors);
          }
        };

        MutationError::InternalServerError(err.into())
      }
      _ => MutationError::InternalServerError(err.into()),
    })?;

  Response::success(UserView::new(user, Some(config), avatar))
}
