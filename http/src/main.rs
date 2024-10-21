mod connection;
mod http;

use connection::handle_connection;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:1337").await.unwrap();
    println!("Now listening on localhost @ 1337...");

    loop {
        let (stream, addr) = listener.accept().await.unwrap();
        tokio::spawn(handle_connection(stream, addr));
    }
}
