use std::fs;

use serde::Deserialize;
use toml::from_str;

#[derive(Deserialize)]
pub struct MaestroConfig {
    server: Server,
    routes: Vec<Route>,
}

#[derive(Deserialize)]
struct Server {
    host: String,
    port: u16,
}

#[derive(Deserialize)]
pub struct Route {
    pub path: String,
    pub target: String,
}

impl MaestroConfig {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let maestro_config: MaestroConfig = from_str(&content)?;
        Ok(maestro_config)
    }

    pub fn server_addr(&self) -> (&str, u16) {
        (self.server.host.as_str(), self.server.port)
    }

    pub fn routes(&self) -> &Vec<Route> {
        &self.routes
    }
}
