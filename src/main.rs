use std::time::Duration;
use std::net::{SocketAddr, Ipv4Addr};
use std::path::Path;
use std::fmt::Debug;
use tokio::{io, fs};
use tokio::net::{UdpSocket, TcpListener};
use hickory_server::server::{ServerFuture, RequestHandler, Request, ResponseHandler, ResponseInfo};
use hickory_server::authority::MessageResponseBuilder;
use hickory_server::proto::op::{Header, ResponseCode, OpCode, MessageType};
use hickory_server::proto::rr::{Record, RecordData, rdata};
use axum::Router;
use axum::routing::get;
use redis::AsyncCommands;
use redis::aio::MultiplexedConnection;
use tracing::info;
use tracing_subscriber::prelude::*;
use tracing_subscriber::filter::LevelFilter;
use serde::{Serialize, Deserialize};

#[tokio::main]
async fn main() -> Result<(), io::Error> {
  tracing_subscriber::registry()
    .with(LevelFilter::INFO)
    .with(tracing_subscriber::fmt::layer())
    .init();

  let config = Config::load("config.toml").await?;

  let redis_client = redis::Client::open(&*config.redis_url).unwrap();
  let redis_con = redis_client
    .get_multiplexed_async_connection()
    .await
    .unwrap();

  let state = State { config, redis_con };

  let mut server = ServerFuture::new(DnsHandler {
    state: state.clone(),
  });
  server.register_socket(UdpSocket::bind(state.config.dns_listen).await?);
  server.register_listener(
    TcpListener::bind(state.config.dns_listen).await?,
    Duration::from_secs(5),
  );
  info!("dns listening on {}", state.config.dns_listen);

  let app = Router::new().route("/", get("HELLO FLOPPA"));
  let http_listener = TcpListener::bind(state.config.http_listen).await?;
  info!("http listening on {}", state.config.http_listen);
  axum::serve(http_listener, app).await
}

#[derive(Clone)]
struct State {
  config: Config,
  redis_con: MultiplexedConnection,
}

impl State {
  async fn get_domain(&self, domain: &str) -> Option<String> {
    self
      .redis_con
      .clone()
      .get(format!("domains:{}", domain))
      .await
      .unwrap()
  }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
struct Config {
  dns_listen: SocketAddr,
  dns_zone: String,
  self_addr: Ipv4Addr,
  http_listen: SocketAddr,
  redis_url: String,
}

impl Default for Config {
  fn default() -> Self {
    Self {
      dns_listen: ([0, 0, 0, 0], 5353).into(),
      dns_zone: "example.com.".to_string(),
      self_addr: [127, 0, 0, 1].into(),
      http_listen: ([0, 0, 0, 0], 3000).into(),
      redis_url: "redis://127.0.0.1/".to_string(),
    }
  }
}

impl Config {
  pub async fn load<P: AsRef<Path> + Debug>(file: P) -> Result<Self, io::Error> {
    let config = match fs::read_to_string(&file).await {
      Ok(contents) => toml::from_str(&contents).map_err(io::Error::other)?,
      Err(err) => match err.kind() {
        io::ErrorKind::NotFound => {
          let default_config = Config::default();
          info!("creating default config {:?}", file);
          fs::write(
            &file,
            toml::to_string_pretty(&default_config).map_err(io::Error::other)?,
          )
          .await?;
          default_config
        }
        _ => return Err(err),
      },
    };
    Ok(config)
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
            let domain = &domain[..domain.len() - 1];
            if !domain.is_empty() {
              if let Some(ip_str) = self.state.get_domain(domain).await {
                ip = ip_str.parse().unwrap();
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
