use std::net::IpAddr;
use hickory_server::server::{RequestHandler, Request, ResponseHandler, ResponseInfo};
use hickory_server::authority::MessageResponseBuilder;
use hickory_server::proto::op::{Header, ResponseCode, OpCode, MessageType};
use hickory_server::proto::rr::{Record, RecordData, rdata};
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
    mut response_handle: R,
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
            let mut ip = self.state.config.self_addr;
            if domain.len() > 1 {
              if let Some(i) = self.state.get_domain(&domain[..domain.len() - 1]).await {
                ip = i;
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
