mod config;

use std::time::Duration;
use std::net::IpAddr;
use tokio::io;
use tokio::net::{UdpSocket, TcpListener};
use hickory_server::server::{ServerFuture, RequestHandler, Request, ResponseHandler, ResponseInfo};
use hickory_server::authority::MessageResponseBuilder;
use hickory_server::proto::op::{Header, ResponseCode, OpCode, MessageType};
use hickory_server::proto::rr::{Record, RecordData, rdata};
use axum::Router;
use axum::routing::post;
use axum::extract::State;
use tower_http::services::{ServeDir, ServeFile};
use sqlx::PgPool;
use argon2::Argon2;
use argon2::password_hash::{SaltString, PasswordHasher};
use argon2::password_hash::rand_core::OsRng;
use tracing::info;
use tracing_subscriber::prelude::*;
use tracing_subscriber::filter::LevelFilter;
use crate::config::Config;

#[tokio::main]
async fn main() -> Result<(), io::Error> {
  tracing_subscriber::registry()
    .with(LevelFilter::DEBUG)
    .with(tracing_subscriber::fmt::layer())
    .init();

  let config = Config::load("config.toml").await?;
  let pg_pool = PgPool::connect(&config.db_url).await.unwrap();
  let state = AppState { config, pg_pool };

  let mut server = ServerFuture::new(DnsHandler {
    state: state.clone(),
  });
  server.register_socket(UdpSocket::bind(state.config.dns_listen).await?);
  server.register_listener(
    TcpListener::bind(state.config.dns_listen).await?,
    Duration::from_secs(5),
  );
  info!("dns listening on {}", state.config.dns_listen);

  let app = Router::new()
    .route("/test", post(test))
    .fallback_service(ServeDir::new("web/dist/").fallback(ServeFile::new("web/dist/index.html")))
    .with_state(state.clone());
  let http_listener = TcpListener::bind(state.config.http_listen).await?;
  info!("http listening on {}", state.config.http_listen);
  axum::serve(http_listener, app).await
}

#[derive(Clone)]
struct AppState {
  config: Config,
  pg_pool: PgPool,
}

impl AppState {
  async fn get_domain(&self, domain: &str) -> Option<IpAddr> {
    sqlx::query_as("SELECT ip FROM domains WHERE name = $1 LIMIT 1")
      .bind(domain)
      .fetch_optional(&self.pg_pool)
      .await
      .unwrap()
      .map(|x: (IpAddr,)| x.0)
  }

  async fn create_user(&self, username: &str, password: &str) -> Result<(), ()> {
    if sqlx::query("SELECT 1 FROM users WHERE username = $1")
      .bind(username)
      .fetch_optional(&self.pg_pool)
      .await
      .unwrap()
      .is_some()
    {
      return Err(());
    }

    let pass_hash = Argon2::default()
      .hash_password(password.as_bytes(), &SaltString::generate(&mut OsRng))
      .unwrap()
      .to_string();

    sqlx::query("INSERT INTO users VALUES ($1, $2, CURRENT_TIMESTAMP)")
      .bind(username)
      .bind(pass_hash)
      .execute(&self.pg_pool)
      .await
      .unwrap();
    Ok(())
  }
}

async fn test(State(state): State<AppState>) {
  state.create_user("floppa", "files").await.unwrap();
}

struct DnsHandler {
  state: AppState,
}

#[async_trait::async_trait]
impl RequestHandler for DnsHandler {
  async fn handle_request<R: ResponseHandler>(
    &self,
    request: &Request,
    mut response_handle: R,
  ) -> ResponseInfo {
    if request.message_type() == MessageType::Query && request.op_code() == OpCode::Query {
      match request.request_info() {
        Ok(info) => {
          let mut ip = self.state.config.self_addr;
          if let Some(domain) = info
            .query
            .name()
            .to_string()
            .strip_suffix(&self.state.config.dns_zone)
          {
            if domain.len() > 1 {
              if let Some(i) = self.state.get_domain(&domain[..domain.len() - 1]).await {
                ip = i;
              }
            }
          }

          let mut header = Header::response_from_request(request.header());
          header.set_authoritative(true);
          response_handle
            .send_response(MessageResponseBuilder::from_message_request(request).build(
              header,
              [&Record::from_rdata(
                info.query.name().into(),
                300,
                match ip {
                  IpAddr::V4(ip) => rdata::A(ip).into_rdata(),
                  IpAddr::V6(ip) => rdata::AAAA(ip).into_rdata(),
                },
              )],
              [],
              [],
              [],
            ))
            .await
            .unwrap()
        }
        Err(_) => send_error(response_handle, request, ResponseCode::FormErr).await,
      }
    } else {
      send_error(response_handle, request, ResponseCode::NotImp).await
    }
  }
}

async fn send_error<R: ResponseHandler>(
  mut response_handle: R,
  request: &Request,
  code: ResponseCode,
) -> ResponseInfo {
  let response_builder = MessageResponseBuilder::from_message_request(request);
  response_handle
    .send_response(response_builder.error_msg(request.header(), code))
    .await
    .unwrap()
}
