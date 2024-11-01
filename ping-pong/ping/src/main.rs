use std::net::SocketAddr;
use tokio::net::UdpSocket;
use tokio::time::{sleep, timeout, Duration, Instant};

const NUM_PINGS: usize = 5;
const TIMEOUT_SEC: u64 = 1;

async fn ping(socket: &UdpSocket, remote_addr: &SocketAddr) -> Result<f64, ()> {
    let mut buf = vec![0; 0xFF];
    let start = Instant::now();

    if let Err(e) = socket.connect(remote_addr).await {
        eprintln!("Connect error: {e}");
        return Err(());
    }
    if let Err(e) = socket.send(b"PING").await {
        eprintln!("Send error: {e}");
        return Err(());
    }

    match timeout(Duration::from_secs(TIMEOUT_SEC), socket.recv(&mut buf)).await {
        Ok(Ok(_)) => {
            if buf.starts_with(b"PONG") {
                let rtt = Instant::now() - start;
                let rtt = rtt.as_secs_f64() + (rtt.subsec_micros() as f64 / 1000.0);
                println!("PONG from {}: RTT={:.2} ms", remote_addr, rtt);

                Ok(rtt)
            } else {
                Err(())
            }
        }
        Ok(Err(e)) => {
            eprintln!("Receive error: {e}");
            Err(())
        }
        Err(_) => {
            println!("Request timed out.");
            Err(())
        }
    }
}

#[tokio::main]
async fn main() {
    let mut total_time = 0.0;

    let Ok(remote_addr) = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:1337".into())
        .parse::<SocketAddr>()
    else {
        eprintln!("Usage: helios-ping <address>:<port>");
        return;
    };

    let local_addr: SocketAddr = "0.0.0.0:0".parse().expect("Infallible");
    let socket = match UdpSocket::bind(local_addr).await {
        Ok(socket) => socket,
        Err(e) => {
            eprintln!("Bind error: {e}");
            return;
        }
    };

    println!("PING {}.\n", remote_addr);
    for _ in 0..NUM_PINGS {
        if let Ok(rtt) = ping(&socket, &remote_addr).await {
            total_time += rtt;
            sleep(Duration::from_secs(TIMEOUT_SEC)).await;
        }
    }

    let average_rtt = total_time / NUM_PINGS as f64;
    println!("\nAverage RTT={:.2} ms", average_rtt);
}
