use std::net::IpAddr;
use axum::{Router, Json};
use axum::routing::{get, post};
use axum::extract::{State, Path, Query, FromRequestParts};
use axum::response::{IntoResponse, Response};
use axum::http::StatusCode;
use axum::http::request::Parts;
use argon2::Argon2;
use argon2::password_hash::{SaltString, PasswordHasher, PasswordHash, PasswordVerifier};
use argon2::password_hash::rand_core::OsRng;
use uuid::Uuid;
use chrono::NaiveDateTime;
use tracing::info;
use serde::{Serialize, Deserialize};
use crate::AppState;

pub fn routes() -> Router<AppState> {
  Router::new()
    .route("/info", get(get_info))
    .route("/auth/user", get(get_user))
    .route("/auth/login", post(login))
    .route("/auth/logout", post(logout))
    .route("/auth/create-account", post(create_account))
    .route("/domains", get(get_domains))
    .route(
      "/domains/{name}",
      post(create_domain).put(update_domain).delete(delete_domain),
    )
}

struct Token(Uuid);

impl<S: Send + Sync> FromRequestParts<S> for Token {
  type Rejection = StatusCode;

  async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
    parts
      .headers
      .get("Authorization")
      .and_then(|h| h.to_str().ok())
      .and_then(|s| s.split(" ").nth(1))
      .and_then(|t| Uuid::parse_str(t).ok())
      .map(Self)
      .ok_or(StatusCode::UNAUTHORIZED)
  }
}

#[derive(Serialize)]
struct Info {
  dns_zone: String,
}

async fn get_info(State(state): State<AppState>) -> Response {
  Json(Info {
    dns_zone: state.config.dns_zone,
  })
  .into_response()
}

#[derive(Serialize)]
struct UserResponse {
  username: String,
  created: NaiveDateTime,
}

async fn get_user(State(state): State<AppState>, Token(token): Token) -> Response {
  let Some(user): Option<(String, NaiveDateTime)> =
    sqlx::query_as("SELECT users.username, users.created FROM users INNER JOIN sessions ON users.username = sessions.username WHERE sessions.id = $1")
      .bind(token)
      .fetch_optional(&state.pg_pool)
      .await
      .unwrap()
  else {
    return StatusCode::UNAUTHORIZED.into_response();
  };
  Json(UserResponse {
    username: user.0,
    created: user.1,
  })
  .into_response()
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

async fn logout(State(state): State<AppState>, Token(token): Token) {
  sqlx::query("DELETE FROM sessions WHERE id = $1")
    .bind(token)
    .execute(&state.pg_pool)
    .await
    .unwrap();
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

#[derive(Serialize)]
struct Domain {
  name: String,
  ipv4: Option<IpAddr>,
  ipv6: Option<IpAddr>,
}

async fn get_domains(State(state): State<AppState>, Token(token): Token) -> Response {
  // probably want stable order
  let domains: Vec<(String,Option<IpAddr>,Option<IpAddr>)> =
    sqlx::query_as("SELECT name, ipv4, ipv6 FROM domains INNER JOIN sessions ON domains.owner = sessions.username WHERE sessions.id = $1")
      .bind(token)
      .fetch_all(&state.pg_pool)
      .await
      .unwrap();

  Json(
    domains
      .into_iter()
      .map(|(name, ipv4, ipv6)| Domain { name, ipv4, ipv6 })
      .collect::<Vec<_>>(),
  )
  .into_response()
}

async fn create_domain(
  State(state): State<AppState>,
  Token(token): Token,
  Path(name): Path<String>,
) -> Response {
  if name.len() > 63 {
    return StatusCode::BAD_REQUEST.into_response();
  }

  // todo extract this
  let Some(username): Option<(String,)> =
    sqlx::query_as("SELECT username FROM sessions WHERE id = $1")
      .bind(token)
      .fetch_optional(&state.pg_pool)
      .await
      .unwrap()
  else {
    return StatusCode::UNAUTHORIZED.into_response();
  };

  if sqlx::query("SELECT 1 FROM domains WHERE name = $1")
    .bind(&name)
    .fetch_optional(&state.pg_pool)
    .await
    .unwrap()
    .is_some()
  {
    return StatusCode::FORBIDDEN.into_response();
  }

  sqlx::query("INSERT INTO domains(name, owner) VALUES ($1, $2)")
    .bind(&name)
    .bind(username.0)
    .execute(&state.pg_pool)
    .await
    .unwrap();
  info!("created new domain '{}'", name);

  StatusCode::OK.into_response()
}

#[derive(Deserialize)]
struct UpdateParams {
  r#type: Option<String>,
}

async fn update_domain(
  State(state): State<AppState>,
  Token(token): Token,
  Path(name): Path<String>,
  Query(params): Query<UpdateParams>,
  body: String,
) -> Response {
  let ip = if body.is_empty() {
    None
  } else {
    let Ok(ip) = body.parse::<IpAddr>() else {
      return StatusCode::BAD_REQUEST.into_response();
    };
    Some(ip)
  };
  let ty = match (params.r#type.as_deref(), ip) {
    (None, None) => return StatusCode::BAD_REQUEST.into_response(),
    (Some("ipv4") | None, Some(IpAddr::V4(_)) | None) => "ipv4",
    (Some("ipv6") | None, Some(IpAddr::V6(_)) | None) => "ipv6",
    _ => return StatusCode::BAD_REQUEST.into_response(),
  };

  if sqlx::query(&format!("UPDATE domains SET {} = $1 FROM sessions WHERE sessions.username = domains.owner AND sessions.id = $2 AND domains.name = $3", ty))
    .bind(ip)
    .bind(token)
    .bind(name)
    .execute(&state.pg_pool)
    .await
    .unwrap()
    .rows_affected() == 0 {
    return StatusCode::FORBIDDEN.into_response();
  }
  StatusCode::OK.into_response()
}

async fn delete_domain(
  State(state): State<AppState>,
  Token(token): Token,
  Path(name): Path<String>,
) -> Response {
  if sqlx::query("DELETE FROM domains USING sessions WHERE sessions.username = domains.owner AND sessions.id = $1 AND domains.name = $2")
    .bind(token)
    .bind(&name)
    .execute(&state.pg_pool)
    .await
    .unwrap()
    .rows_affected() == 0 {
    
    return StatusCode::FORBIDDEN.into_response();
  }
  info!("deleted domain '{}'", name);
  StatusCode::OK.into_response()
}
