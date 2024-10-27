use crate::config::Config;
use crate::http::*;
use crate::response::*;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    time::{timeout, Duration},
};

/// Sends a response message to a client.
async fn send_response(
    stream: &mut BufReader<impl AsyncWriteExt + AsyncReadExt + Unpin>,
    response: HttpMessage,
) -> std::io::Result<()> {
    stream.write_all(&Vec::from(response)).await.map_err(|e| {
        eprintln!("Error writing to stream: {e}");
        e
    })
}

/// Both creates an HTTP error response and sends it over connection.
async fn create_and_send_err_response(
    config: &Config,
    stream: &mut BufReader<impl AsyncWriteExt + AsyncReadExt + Unpin>,
    status_code: HttpStatusCode,
) -> std::io::Result<()> {
    let response = create_error_response(config, status_code).await;
    send_response(stream, response).await
}

/// Read in an HTTP header.
async fn read_header(
    config: &Config,
    stream: &mut BufReader<impl AsyncWriteExt + AsyncReadExt + Unpin>,
) -> Result<String, ()> {
    let mut header = String::new();
    let read_timeout = Duration::from_secs(5);

    // Read and parse header
    while !header.ends_with("\r\n\r\n") {
        match timeout(read_timeout, stream.read_line(&mut header)).await {
            Ok(Ok(0)) => {
                println!("Connection closed by client...");
                return Err(());
            }
            Ok(Err(e)) => {
                eprintln!("Error reading from stream: {e}");
                let _ = create_and_send_err_response(
                    config,
                    stream,
                    HttpStatusCode::InternalServorError,
                )
                .await;
                return Err(());
            }
            Err(_) => {
                println!("Timeout, closing connection...");
                let _ =
                    create_and_send_err_response(config, stream, HttpStatusCode::RequestTimeout)
                        .await;
                return Err(());
            }
            _ => (),
        }

        if header.len() > config.max_header_len {
            let _ =
                create_and_send_err_response(config, stream, HttpStatusCode::ContentTooLarge).await;
            return Err(());
        }
    }

    Ok(header)
}

/// Read in an HTTP body.
async fn read_body(
    config: &Config,
    stream: &mut BufReader<impl AsyncWriteExt + AsyncReadExt + Unpin>,
    length: usize,
) -> Result<Vec<u8>, ()> {
    let read_timeout = Duration::from_secs(config.max_timeout);
    let mut body = vec![0; length];

    match timeout(read_timeout, stream.read_exact(&mut body)).await {
        Ok(Ok(0)) => {
            println!("Connection closed by client...");
            Err(())
        }
        Ok(Err(e)) => {
            eprintln!("Error reading from stream: {e}");
            let _ =
                create_and_send_err_response(config, stream, HttpStatusCode::InternalServorError)
                    .await;
            Err(())
        }
        Err(_) => {
            println!("Timeout, closing connection...");
            let _ =
                create_and_send_err_response(config, stream, HttpStatusCode::RequestTimeout).await;
            Err(())
        }
        _ => Ok(body),
    }
}

/// Handles an incoming connection from a client.
pub async fn handle_connection(
    config: &Config,
    stream: impl AsyncWriteExt + AsyncReadExt + Unpin,
    addr: SocketAddr,
    conn_sem: Arc<Semaphore>,
) {
    let mut stream = BufReader::new(stream);
    if conn_sem.try_acquire().is_err() {
        println!("Server overloaded, ignoring connection.");
        let _ =
            create_and_send_err_response(config, &mut stream, HttpStatusCode::ServiceUnavailable)
                .await;
        return;
    }

    println!("Handling connection from {addr}...");

    // Loop until timeout or EOF (unless keep-alive is disabled)
    'connection: loop {
        // Read and parse header
        let Ok(header) = read_header(config, &mut stream).await else {
            let _ = create_and_send_err_response(
                config,
                &mut stream,
                HttpStatusCode::InternalServorError,
            )
            .await;
            break 'connection;
        };

        // If header malformed, send error response and close connection
        let header = match header.parse::<HttpHeader>() {
            Ok(header) => header,
            Err(e) => {
                let status_code = match e {
                    Error::UnsupportedMethod => HttpStatusCode::NotImplemented,
                    Error::UnsupportedVersion => HttpStatusCode::HTTPVersionNotSupported,
                    _ => HttpStatusCode::BadRequest,
                };

                let _ = create_and_send_err_response(config, &mut stream, status_code).await;
                break 'connection;
            }
        };

        // Client sent us a response? Ignore.
        if !header.is_request() {
            let _ =
                create_and_send_err_response(config, &mut stream, HttpStatusCode::BadRequest).await;
            break 'connection;
        };

        // If request contains body, read it
        let body = if let Some(length) = header.field_lines.get("content-length") {
            let Ok(length) = length.parse() else {
                let _ =
                    create_and_send_err_response(config, &mut stream, HttpStatusCode::BadRequest)
                        .await;
                break 'connection;
            };

            if length > config.max_body_len {
                let _ = create_and_send_err_response(
                    config,
                    &mut stream,
                    HttpStatusCode::ContentTooLarge,
                )
                .await;
                break 'connection;
            }
            match read_body(config, &mut stream, length).await {
                Ok(body) => Some(body),
                Err(_) => break 'connection,
            }
        } else {
            None
        };

        // Perform what is asked from request
        let request = HttpMessage { header, body };
        let response = process_request(config, &request).await;
        if send_response(&mut stream, response).await.is_err() || !request.header.is_persistent() {
            break 'connection;
        }
    }
}
