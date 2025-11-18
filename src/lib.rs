pub mod config;
pub mod error;

use std::net::SocketAddr;
use std::sync::Arc;

use axum::body::Body;
use axum::extract::{ConnectInfo, Request, State};
use axum::http::{StatusCode, Response};
use axum::response::IntoResponse;
use axum::middleware::Next;

use crate::error::ServerError;
use crate::config::Config;


const CONFIG_PATH: &str = "FORGEJO_PROXY_CONFIG";

pub fn get_config() -> Result<Config, ServerError> {
    let default_config_path: &'static str = "/etc/forgejo-proxy/config.toml";

    let config_path = match std::env::var(CONFIG_PATH) {
        Ok(config_path) => {
            tracing::debug!(
                "Config path found from env variable {}, leading to {}",
                CONFIG_PATH,
                config_path,
            );
            config_path
        },
        Err(e) => {
            tracing::error!("ERROR: {e:#?}");
            tracing::warn!(
                "{} env variable not set or inaccessable, defaulting to {}",
                CONFIG_PATH,
                default_config_path,
            );
            default_config_path.to_string()
        },
    };

    let content = std::fs::read_to_string(&config_path)?;
    tracing::debug!("Successfully read config file");

    let config = toml::from_str::<Config>(&content)?;
    tracing::debug!("Successfully parsed TOML config");
    Ok(config)
}

pub async fn check_ip_whitelist(
    State(config): State<Arc<Config>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    req: Request<Body>,
    next: Next,
) -> Response<Body> {
    let ip = addr.ip();

    if config.server.whitelist.contains(&ip) {
        next.run(req).await.into_response()
    } else {
        tracing::warn!("BLOCKED request from unauthorized IP: {ip}");
        StatusCode::FORBIDDEN.into_response()
    }
}

pub async fn log_traffic(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    req: Request<Body>,
    next: Next,
) -> Response<Body> {
    let method = req.method().clone();
    let uri = req.uri().clone();
    let path = uri.path();

    let is_asset = path.starts_with("/assets/")
        || path.ends_with(".css")
        || path.ends_with(".js")
        || path.ends_with(".png")
        || path.ends_with(".svg")
        || path.ends_with(".ico")
        || path.ends_with(".woff2");

    let response = next.run(req).await;

    if !is_asset {
        tracing::info!(
            "\nACCESS: {method} {uri}\nFROM {addr} -> status: {}",
            response.status()
        );
    }
    response
}
