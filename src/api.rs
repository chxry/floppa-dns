use axum::Router;
use axum::routing::{get, post};
use axum::extract::{State, Json};
use axum::response::{IntoResponse, Response};
use axum::http::{StatusCode, HeaderMap};
use argon2::Argon2;
use argon2::password_hash::{SaltString, PasswordHasher, PasswordHash, PasswordVerifier};
use argon2::password_hash::rand_core::OsRng;
use uuid::Uuid;
use tracing::info;
use serde::Deserialize;
use crate::AppState;

pub fn routes() -> Router<AppState> {
  Router::new()
    .route("/auth/user", get(user))
    .route("/auth/login", post(login))
    .route("/auth/create-account", post(create_account))
}

async fn user(State(state): State<AppState>, headers: HeaderMap) -> Response {
  let Some(token) = headers
    .get("Authorization")
    .and_then(|h| h.to_str().ok())
    .and_then(|s| s.split(" ").nth(1))
    .and_then(|t| Uuid::parse_str(t).ok())
  else {
    return StatusCode::BAD_REQUEST.into_response();
  };

  let Some(username): Option<(String,)> =
    sqlx::query_as("SELECT username FROM sessions WHERE id = $1")
      .bind(token)
      .fetch_optional(&state.pg_pool)
      .await
      .unwrap()
  else {
    return StatusCode::UNAUTHORIZED.into_response();
  };
  username.into_response()
}

#[derive(Deserialize)]
struct UserDetails {
  username: String,
  password: String,
}

async fn login(State(state): State<AppState>, Json(body): Json<UserDetails>) -> Response {
  let Some(pass_hash): Option<(String,)> =
    sqlx::query_as("SELECT pass_hash FROM users WHERE username = $1")
      .bind(&body.username)
      .fetch_optional(&state.pg_pool)
      .await
      .unwrap()
  else {
    return StatusCode::UNAUTHORIZED.into_response();
  };

  if Argon2::default()
    .verify_password(
      body.password.as_bytes(),
      &PasswordHash::new(&pass_hash.0).unwrap(),
    )
    .is_ok()
  {
    state
      .create_session(&body.username)
      .await
      .to_string()
      .into_response()
  } else {
    StatusCode::UNAUTHORIZED.into_response()
  }
}

async fn create_account(State(state): State<AppState>, Json(body): Json<UserDetails>) -> Response {
  if body.username.len() > 64 {
    return StatusCode::BAD_REQUEST.into_response();
  }
  if sqlx::query("SELECT 1 FROM users WHERE username = $1")
    .bind(&body.username)
    .fetch_optional(&state.pg_pool)
    .await
    .unwrap()
    .is_some()
  {
    return StatusCode::FORBIDDEN.into_response();
  }

  let pass_hash = Argon2::default()
    .hash_password(body.password.as_bytes(), &SaltString::generate(&mut OsRng))
    .unwrap()
    .to_string();

  sqlx::query("INSERT INTO users VALUES ($1, $2, CURRENT_TIMESTAMP)")
    .bind(&body.username)
    .bind(pass_hash)
    .execute(&state.pg_pool)
    .await
    .unwrap();
  info!("created user '{}'", body.username);

  state
    .create_session(&body.username)
    .await
    .to_string()
    .into_response()
}
