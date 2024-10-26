mod cgi;
mod connection;
mod http;
mod response;

use connection::handle_connection;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Semaphore;

const MAX_CONNECTIONS: usize = 10;

#[tokio::main]
async fn main() {
    let conn_sem = Arc::new(Semaphore::new(MAX_CONNECTIONS));
    let listener = TcpListener::bind("127.0.0.1:1337").await.unwrap();
    println!("Now listening on localhost @ 1337...");

    loop {
        let (stream, addr) = listener.accept().await.unwrap();
        tokio::spawn(handle_connection(stream, addr, Arc::clone(&conn_sem)));
    }
}
