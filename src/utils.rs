use bytes::Bytes;
use http_body_util::Full;
use hyper::{Response, StatusCode};

// Helper function: generate error response
pub fn error_response(status: StatusCode, message: &str) -> Response<Full<Bytes>> {
    Response::builder()
        .status(status)
        .body(Full::new(Bytes::from(message.to_string())))
        .unwrap()
}
