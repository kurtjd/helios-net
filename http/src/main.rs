mod cgi;
mod config;
mod connection;
mod http;
mod response;

use config::Config;
use connection::handle_connections;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Semaphore;

#[tokio::main]
async fn main() {
    let config = std::env::args()
        .nth(1)
        .map_or_else(Config::default, |path| {
            Config::from_file(Path::new(&path)).unwrap_or_else(|_| {
                eprintln!("Error: Could not retrieve configuration settings.");
                std::process::exit(1);
            })
        });

    // We want config to have static lifetime so it can be shared among tokio tasks
    let config = Box::leak(Box::new(config));
    let conn_sem = Arc::new(Semaphore::new(config.max_connections));

    // Will only return on unrecoverable error
    handle_connections(config, conn_sem).await;
}
