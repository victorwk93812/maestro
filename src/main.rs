use std::sync::Arc;

use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;
use log::{error, info};
use tokio::net::TcpListener;

pub mod config;
pub mod handler;
pub mod utils;

use crate::config::MaestroConfig;
use crate::handler::handle_proxy_request;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Quick start: use default initialization
    colog::init();

    let config = MaestroConfig::from_file("maestro.toml").unwrap();

    let addr = MaestroConfig::server_addr(&config);
    let listener = TcpListener::bind(addr).await?;
    let routes = MaestroConfig::routes(&config);

    info!(
        "✅ Reverse Proxy is activated at http://{}:{}",
        addr.0, addr.1
    );

    let routes_string = routes
        .iter()
        .map(|r| format!("{} -> {}", r.path, r.target))
        .collect::<Vec<String>>()
        .join("\n");

    info!("📄 Loaded Routes:\n{}", routes_string);

    info!("🔄 Waiting for incoming connections...");

    let client = Client::builder(TokioExecutor::new()).build_http::<hyper::body::Incoming>();
    let shared_client = Arc::new(client);

    loop {
        let (stream, client_addr) = listener.accept().await?;
        let client_clone = Arc::clone(&shared_client);

        tokio::task::spawn(async move {
            let service = service_fn(move |req| {
                handle_proxy_request(req, Arc::clone(&client_clone), client_addr)
            });

            if let Err(err) = http1::Builder::new()
                .serve_connection(hyper_util::rt::TokioIo::new(stream), service)
                .await
            {
                error!("❌ Service connection error: {:?}", err);
            }
        });
    }
}
