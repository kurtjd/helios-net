mod http_parser;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    //time::{sleep, Duration},
};

async fn handle_connection(mut stream: TcpStream) {
    println!("Connection received...");
    let mut buf = vec![0; 1024];
    let _ = stream.read(&mut buf).await.unwrap();

    println!("{}", String::from_utf8(buf).unwrap());

    stream
        .write_all("HTTP/1.1 200 OK\r\n\r\nHack the planet!\n".as_bytes())
        .await
        .unwrap();
}

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:1337").await.unwrap();
    println!("Now listening on localhost @ 1337...");

    loop {
        let (stream, _) = listener.accept().await.unwrap();
        tokio::spawn(handle_connection(stream));
    }
}
