use std::net::IpAddr;

use serde::Deserialize;


#[derive(Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub proxy: ProxyConfig,
    pub tls: TlsConfig,
}

#[derive(Deserialize)]
pub struct ServerConfig {
    pub listen_addr: IpAddr,
    pub listen_port: u16,
    pub whitelist: Vec<IpAddr>,
}

#[derive(Deserialize)]
pub struct ProxyConfig {
    pub protocol: String, // "http" or "https"
    pub target_addr: IpAddr,
    pub target_port: u16,
}

#[derive(Deserialize)]
pub struct TlsConfig {
    pub cert_path: String,
    pub key_path: String,
}
