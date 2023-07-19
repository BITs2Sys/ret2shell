use std::net::IpAddr;

use axum::http::Request;

use crate::controller::forwarded::{
    maybe_connect_info, maybe_forwarded, maybe_x_forwarded_for, maybe_x_real_ip,
};

pub fn get_client_ip_address_from_request<B: Send>(request: &Request<B>) -> Option<IpAddr> {
    let headers = request.headers();
    maybe_x_forwarded_for(headers)
        .or_else(|| maybe_x_real_ip(headers))
        .or_else(|| maybe_forwarded(headers))
        .or_else(|| maybe_connect_info(request))
}
