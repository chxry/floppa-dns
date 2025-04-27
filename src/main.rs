mod config;
mod dns;
mod api;

use std::time::Duration;
use std::net::IpAddr;
use tokio::io;
use tokio::net::{UdpSocket, TcpListener};
use hickory_server::server::ServerFuture;
use axum::Router;
use tower_http::services::{ServeDir, ServeFile};
use sqlx::PgPool;
use uuid::Uuid;
use tracing::info;
use tracing_subscriber::prelude::*;
use tracing_subscriber::filter::LevelFilter;
use crate::config::Config;
use crate::dns::DnsHandler;

#[tokio::main]
async fn main() -> Result<(), io::Error> {
  tracing_subscriber::registry()
    .with(LevelFilter::INFO)
    .with(tracing_subscriber::fmt::layer())
    .init();

  let config = Config::load("config.toml").await?;
  let pg_pool = PgPool::connect(&config.db_url).await.unwrap();
  let state = AppState { config, pg_pool };

  let mut server = ServerFuture::new(DnsHandler::new(&state));
  server.register_socket(UdpSocket::bind(state.config.dns_listen).await?);
  server.register_listener(
    TcpListener::bind(state.config.dns_listen).await?,
    Duration::from_secs(5),
  );
  info!("dns listening on {}", state.config.dns_listen);

  let app = Router::new()
    .nest("/api", api::routes())
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

  async fn create_session(&self, username: &str) -> Uuid {
    let session_id = Uuid::new_v4();
    sqlx::query("INSERT INTO sessions VALUES ($1, $2)")
      .bind(session_id)
      .bind(username)
      .execute(&self.pg_pool)
      .await
      .unwrap();
    info!("created session for '{}'", username);
    session_id
  }
}
