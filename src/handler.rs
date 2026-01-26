use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::header::{HOST, HeaderValue};
use hyper::{Request, Response, StatusCode, Uri};
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;
use tokio::time::timeout;

use crate::utils::error_response;

const BACKEND_URL: &str = "127.0.0.1:3000";

pub async fn handle_proxy_request(
    mut req: Request<hyper::body::Incoming>,
    client: Arc<Client<HttpConnector, hyper::body::Incoming>>,
    client_addr: SocketAddr,
) -> Result<Response<Full<Bytes>>, std::convert::Infallible> {
    let client_ip = client_addr.ip().to_string();
    let x_forwarded_for = match req.headers().get("X-Forwarded-For") {
        Some(existing) => format!("{}, {}", existing.to_str().unwrap_or(""), &client_ip),
        None => client_ip.clone(),
    };
    req.headers_mut().insert(
        "X-Forwarded-For",
        HeaderValue::from_str(&x_forwarded_for).unwrap(),
    );
    req.headers_mut()
        .insert("X-Real-IP", HeaderValue::from_str(&client_ip).unwrap());

    let path_query = req
        .uri()
        .path_and_query()
        .map(|pq| pq.as_str())
        .unwrap_or("/");
    let new_uri = format!("http://{}{}", BACKEND_URL, path_query)
        .parse::<Uri>()
        .unwrap();
    *req.uri_mut() = new_uri;

    req.headers_mut().insert(HOST, BACKEND_URL.parse().unwrap());

    let request_timeout = Duration::from_secs(5);

    match timeout(request_timeout, client.request(req)).await {
        Ok(Ok(resp)) => {
            let (parts, body) = resp.into_parts();

            match body.collect().await {
                Ok(collected) => Ok(Response::from_parts(parts, Full::new(collected.to_bytes()))),
                Err(_) => Ok(error_response(
                    StatusCode::BAD_GATEWAY,
                    "Failed to read backend data",
                )),
            }
        }
        // Backend connection error (e.g., port 3000 not open)
        Ok(Err(e)) => {
            eprintln!("Proxy request failed: {:?}", e);
            Ok(error_response(
                StatusCode::BAD_GATEWAY,
                "Unable to connect to backend server (502 Bad Gateway)",
            ))
        }
        // Timeout occurred
        Err(_) => Ok(error_response(
            StatusCode::GATEWAY_TIMEOUT,
            "Backend server response timed out (504 Gateway Timeout)",
        )),
    }
}
