use std::time::Duration;
use std::net::SocketAddr;
use std::path::Path;
use std::fmt::Debug;
use tokio::{io, fs};
use tokio::net::{UdpSocket, TcpListener};
use hickory_server::server::{ServerFuture, RequestHandler, Request, ResponseHandler, ResponseInfo};
use hickory_server::authority::MessageResponseBuilder;
use hickory_server::proto::op::{Header, ResponseCode, OpCode, MessageType};
use hickory_server::proto::rr::{Record, RecordData, rdata};
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
  let redis_con = redis_client.get_connection().unwrap();

  let mut server = ServerFuture::new(DnsHandler {
    config: config.clone(),
  });

  server.register_socket(UdpSocket::bind(config.dns_listen).await?);
  info!("udp listening on {:?}", config.dns_listen);

  server.register_listener(
    TcpListener::bind(config.dns_listen).await?,
    Duration::from_secs(5),
  );
  info!("tcp listening on {:?}", config.dns_listen);

  std::future::pending().await
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
struct Config {
  dns_listen: SocketAddr,
  dns_zone: String,
  http_listen: SocketAddr,
  redis_url: String,
}

impl Default for Config {
  fn default() -> Self {
    Self {
      dns_listen: ([0, 0, 0, 0], 5353).into(),
      dns_zone: "example.com".to_string(),
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
  config: Config,
}

#[async_trait::async_trait]
impl RequestHandler for DnsHandler {
  async fn handle_request<R: ResponseHandler>(
    &self,
    request: &Request,
    response_handle: R,
  ) -> ResponseInfo {
    if request.message_type() == MessageType::Query && request.op_code() == OpCode::Query {
      match request.request_info() {
        Ok(info) => match info
          .query
          .name()
          .to_string()
          .strip_suffix(&self.config.dns_zone)
        {
          Some(sub) if !sub.is_empty() => {
            tracing::info!("handle {:?}", &sub[..sub.len() - 1]);
            send_record(
              response_handle,
              request,
              Record::from_rdata(
                info.query.name().into(),
                300,
                rdata::A::new(4, 5, 6, 7).into_rdata(),
              ),
            )
            .await
          }
          _ => {
            send_record(
              response_handle,
              request,
              Record::from_rdata(
                info.query.name().into(),
                300,
                rdata::A::new(1, 2, 3, 4).into_rdata(),
              ),
            )
            .await
          }
        },
        Err(_) => send_error(response_handle, request, ResponseCode::FormErr).await,
      }
    } else {
      send_error(response_handle, request, ResponseCode::NotImp).await
    }
  }
}

async fn send_record<R: ResponseHandler>(
  mut response_handle: R,
  request: &Request,
  record: Record,
) -> ResponseInfo {
  let mut header = Header::response_from_request(request.header());
  header.set_authoritative(true);
  response_handle
    .send_response(MessageResponseBuilder::from_message_request(request).build(
      header,
      [&record],
      [],
      [],
      [],
    ))
    .await
    .unwrap()
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
