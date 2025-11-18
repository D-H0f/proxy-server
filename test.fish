#!/usr/bin/fish

cargo build --release 

set -x FORGEJO_PROXY_CONFIG "config.toml"

set -x RUST_LOG "info,proxy_server=info,tower_http=info"

target/release/proxy-server
