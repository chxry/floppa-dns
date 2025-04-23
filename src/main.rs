mod config;
mod web;

use std::time::Duration;
use std::net::Ipv4Addr;
use tokio::io;
use tokio::net::{UdpSocket, TcpListener};
use hickory_server::server::{ServerFuture, RequestHandler, Request, ResponseHandler, ResponseInfo};
use hickory_server::authority::MessageResponseBuilder;
use hickory_server::proto::op::{Header, ResponseCode, OpCode, MessageType};
use hickory_server::proto::rr::{Record, RecordData, rdata};
use axum::Router;
use axum::routing::get;
use tower_http::services::ServeDir;
use sqlx::PgPool;
use tracing::info;
use tracing_subscriber::prelude::*;
use tracing_subscriber::filter::LevelFilter;
use crate::config::Config;
use crate::web::{home, notfound};

#[tokio::main]
async fn main() -> Result<(), io::Error> {
  tracing_subscriber::registry()
    .with(LevelFilter::DEBUG)
    .with(tracing_subscriber::fmt::layer())
    .init();

  let config = Config::load("config.toml").await?;
  let pg_pool = PgPool::connect(&config.db_url).await.unwrap();
  let state = State { config, pg_pool };

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
    .route("/", get(home))
    .nest_service("/static", ServeDir::new("static"))
    .fallback(notfound);
  let http_listener = TcpListener::bind(state.config.http_listen).await?;
  info!("http listening on {}", state.config.http_listen);
  axum::serve(http_listener, app).await
}

#[derive(Clone)]
struct State {
  config: Config,
  pg_pool: PgPool,
}

impl State {
  async fn get_domain(&self, domain: &str) -> Option<Ipv4Addr> {
    let rec: Option<(String,)> = sqlx::query_as("SELECT ip FROM domains WHERE name = $1 LIMIT 1")
      .bind(domain)
      .fetch_optional(&self.pg_pool)
      .await
      .unwrap();
    rec.and_then(|(ip_str,)| ip_str.parse().ok())
  }
}

struct DnsHandler {
  state: State,
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
                rdata::A(ip).into_rdata(),
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
