use std::net::SocketAddr;
use tokio::net::UdpSocket;

#[tokio::main]
async fn main() {
    let Ok(addr) = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:1337".into())
        .parse::<SocketAddr>()
    else {
        eprintln!("Usage: helios-pong <address>:<port>");
        return;
    };

    let socket = match UdpSocket::bind(addr).await {
        Ok(socket) => socket,
        Err(e) => {
            eprintln!("Bind error: {e}");
            return;
        }
    };

    println!("Listening for PING on {}...\n", addr);
    loop {
        let mut buf = vec![0; 0xFF];
        let addr = match socket.recv_from(&mut buf).await {
            Ok((_, addr)) => addr,
            Err(e) => {
                eprintln!("Receive error: {e}");
                continue;
            }
        };

        if buf.starts_with(b"PING") {
            println!("PING from {}", addr);

            if let Err(e) = socket.send_to(b"PONG", addr).await {
                eprintln!("Send error: {e}");
            }
        }
    }
}
