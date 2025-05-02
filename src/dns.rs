use std::net::IpAddr;
use hickory_server::server::{RequestHandler, Request, ResponseHandler, ResponseInfo};
use hickory_server::authority::MessageResponseBuilder;
use hickory_server::proto::op::{Header, ResponseCode, OpCode, MessageType};
use hickory_server::proto::rr::{Record, RecordData, RecordType, LowerName, rdata};
use crate::AppState;

pub struct DnsHandler {
  state: AppState,
}

impl DnsHandler {
  pub fn new(state: &AppState) -> Self {
    Self {
      state: state.clone(),
    }
  }
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
          .strip_suffix(&self.state.config.dns_zone)
        {
          Some(domain) => {
            let (ty, mut ip) = match info.query.query_type() {
              RecordType::A => ("ipv4", self.state.config.self_addr_v4.map(|x| x.into())),
              RecordType::AAAA => ("ipv6", self.state.config.self_addr_v6.map(|x| x.into())),
              _ => return send_record(response_handle, request, info.query.name(), None).await,
            };

            if domain.len() > 1 {
              let row = sqlx::query_as(&format!("SELECT {} FROM domains WHERE name = $1", ty))
                .bind(&domain[..domain.len() - 1])
                .fetch_optional(&self.state.pg_pool)
                .await
                .unwrap();
              if let Some((Some(i),)) = row {
                ip = i;
              }
            }

            send_record(response_handle, request, info.query.name(), ip).await
          }
          None => send_error(response_handle, request, ResponseCode::Refused).await,
        },
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

async fn send_record<R: ResponseHandler>(
  mut response_handle: R,
  request: &Request,
  name: &LowerName,
  ip: Option<IpAddr>,
) -> ResponseInfo {
  let mut header = Header::response_from_request(request.header());
  header.set_authoritative(true);
  let builder = MessageResponseBuilder::from_message_request(request);
  match ip {
    Some(ip) => response_handle
      .send_response(builder.build(
        header,
        [&Record::from_rdata(
          name.into(),
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
      .unwrap(),
    None => {
      return response_handle
        .send_response(builder.build_no_records(header))
        .await
        .unwrap();
    }
  }
}
