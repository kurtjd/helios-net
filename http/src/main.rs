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

use rustls::pki_types::pem::PemObject;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use tokio_rustls::{rustls, TlsAcceptor};

async fn handle_https(config: &'static Config, conn_sem: Arc<Semaphore>) {
    // HTTP
    let addr = format!("{}:{}", config.ip, config.port_http);
    let listener = TcpListener::bind(&addr).await.unwrap();
    println!("Listening on {addr} for HTTP...");

    // HTTPS
    let certs = CertificateDer::pem_file_iter(format!("{}/crypt/public.pem", config.server_root))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let key =
        PrivateKeyDer::from_pem_file(format!("{}/crypt/private.pem", config.server_root)).unwrap();
    let config_s = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .unwrap();
    let acceptor = TlsAcceptor::from(Arc::new(config_s));
    let addr_s = format!("{}:{}", config.ip, config.port_https);
    let listener_s = TcpListener::bind(&addr_s).await.unwrap();
    println!("Listening on {addr_s} for HTTPS...");

    // Connection loop
    loop {
        tokio::select! {
            connection = listener.accept() => {
                let (stream, addr) = connection.unwrap();
                tokio::spawn(handle_connection(
                    config,
                    stream,
                    addr,
                    Arc::clone(&conn_sem),
                ));
            }
            connection = listener_s.accept() => {
                let (stream, addr) = connection.unwrap();
                let stream = acceptor.accept(stream).await.unwrap();
                tokio::spawn(handle_connection(
                    config,
                    stream,
                    addr,
                    Arc::clone(&conn_sem),
                ));
            }
        }
    }
}

async fn handle_http_only(config: &'static Config, conn_sem: Arc<Semaphore>) {
    let addr = format!("{}:{}", config.ip, config.port_http);
    let listener = TcpListener::bind(&addr).await.unwrap();
    println!("Listening on {addr} for HTTP...");

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

    if config.https_enabled {
        handle_https(config, conn_sem).await
    } else {
        handle_http_only(config, conn_sem).await
    }
}
