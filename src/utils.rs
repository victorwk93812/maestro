use bytes::Bytes;
use http_body_util::Full;
use hyper::{Response, StatusCode};
use log::{info, warn};

// Helper function: generate error response
pub fn error_response(
    status: StatusCode,
    source: &str,
    destination: &str,
    reason: &std::io::Error,
) -> Response<Full<Bytes>> {
    warn!("[FAILED] {}: {} -> {}\n{}", status, source, destination, reason);

    Response::builder()
        .status(status)
        .body(Full::new(Bytes::from(source.to_string())))
        .unwrap()
}

pub fn success_response(
    status: StatusCode,
    source: &str,
    destination: &str,
    response_body: &str,
) -> Response<Full<Bytes>> {
    info!("[SUCCESS] {}, {} -> {}", status, source, destination);

    Response::builder()
        .status(status)
        .body(Full::new(Bytes::from(response_body.to_string())))
        .unwrap()
}
