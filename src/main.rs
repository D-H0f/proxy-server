use std::{
    net::SocketAddr, sync::Arc,
};

use axum::{Router, body::Body, extract::{ConnectInfo, Request}, middleware};
//use axum::extract::ConnectInfo;
use axum_reverse_proxy::ReverseProxy;
use axum_server::tls_rustls::RustlsConfig;
use proxy_server::{check_ip_whitelist, config::Config, error::ServerError, get_config, log_traffic};
use tower_http::trace::TraceLayer;


#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    tracing::info!("Setting up forgejo-proxy");

    let config: Config = match get_config() {
        Ok(config) => config,
        Err(e) => {
            match &e {
                ServerError::TomlParseError(e) => {
                    tracing::error!("ERROR: Failed to parse TOML config file: {e}");
                },
                ServerError::IOError(e) => {
                    tracing::error!("ERROR: Failed to read from file: {e:#?}");
                },
            }
            panic!("ERROR: '{e:#?}");
        },
    };
    let config = Arc::new(config);
    let target_url = format!(
        "{}://{}:{}",
        config.proxy.protocol,
        config.proxy.target_addr,
        config.proxy.target_port,
    );

    let proxy = ReverseProxy::new("/".to_string(), target_url.clone());

    let app: Router = proxy.into();

    let app = app
        .layer(middleware::from_fn_with_state(config.clone(), check_ip_whitelist))
        .layer(middleware::from_fn(log_traffic))
        .layer(
            TraceLayer::new_for_http().make_span_with(|request: &Request<Body>|{
                let remote_addr = request
                    .extensions()
                    .get::<ConnectInfo<SocketAddr>>()
                    .map(|ConnectInfo(addr)| addr.to_string())
                    .unwrap_or_else(|| "unknown".to_string());
                tracing::info_span!(
                    "request",
                    method = %request.method(),
                    uri = %request.uri(),
                    remote_addr = %remote_addr,
                )
            }),
        );

    let tls_config =
        match RustlsConfig::from_pem_file(&config.tls.cert_path, &config.tls.key_path).await {
            Ok(config) => {
            tracing::debug!("Loaded TLS config sucessfully");
            config
        },
            Err(e) => {
                tracing::error!("ERROR: Failed to load TLS certificates: '{e:?}'");
                panic!("Failed to load TLS: '{e:?}'");
            }
        };

    let addr = SocketAddr::new(config.server.listen_addr, config.server.listen_port);

    tracing::info!(
        "Proxy listening on https://{}, proxying to {}",
        addr,
        target_url,
    );

    if let Err(e) = axum_server::bind_rustls(addr, tls_config)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
    {
        tracing::error!("ERROR: Server failed to run: {e:#?}");
        panic!("ERROR: Server failed to run: {e:#?}");
    }
}
