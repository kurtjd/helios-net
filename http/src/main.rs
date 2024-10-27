/* TODO Overall:
 * -Add HTTPS support
 * -Add module doc strings
 */

mod cgi;
mod config;
mod connection;
mod http;
mod response;

use config::Config;
use connection::handle_connection;
use std::path::Path;
use std::sync::Arc;
use tokio::net::TcpListener;
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

    let addr = &format!("{}:{}", config.ip, config.port_http);
    let listener = TcpListener::bind(addr).await.unwrap();
    println!("Now listening on {addr}...");

    let conn_sem = Arc::new(Semaphore::new(config.max_connections));
    loop {
        let (stream, addr) = listener.accept().await.unwrap();
        tokio::spawn(handle_connection(
            config,
            stream,
            addr,
            Arc::clone(&conn_sem),
        ));
    }
}
