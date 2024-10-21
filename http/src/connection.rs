use crate::http::*;
use std::net::SocketAddr;
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
    time::{timeout, Duration},
};

/// Creates a response with server-specific fields.
fn create_response(
    status_code: HttpStatusCode,
    body: Option<Vec<u8>>,
    send_body: bool,
) -> HttpMessage {
    let body = body.unwrap_or_default();

    let field_lines = [
        ("Server", "Helios/13.37"),
        ("Content-Length", &body.len().to_string()),
        ("Date", "Sat, 20 Oct 2024 13:37:00 GMT"),
        ("Connection", "keep-alive"),
    ];
    HttpMessage::new_response(
        HttpVersion::HTTP11,
        status_code,
        &field_lines,
        send_body.then_some(body).unwrap_or_default(),
    )
}

/// Sends a response message to a client.
async fn send_response(
    stream: &mut BufReader<TcpStream>,
    response: HttpMessage,
) -> std::io::Result<()> {
    stream.write_all(&Vec::from(response)).await.map_err(|e| {
        eprintln!("Error writing to stream: {e}");
        e
    })
}

async fn create_and_send_response(
    stream: &mut BufReader<TcpStream>,
    status_code: HttpStatusCode,
    body: Option<Vec<u8>>,
    send_body: bool,
) -> std::io::Result<()> {
    let response = create_response(status_code, body, send_body);
    send_response(stream, response).await
}

/// Handle GET request.
async fn handle_get(_request: &HttpMessage) -> HttpMessage {
    // Canonicalize target (e.g. %20 -> space)
    // Normalize target to prevent directory traversal attack
    // Check if file exists, if not, return 404
    // Check if PHP file, if so, pass file to PHP interpreter
    // Load file into buffer
    // Send 200 OK with body
    let body = b"Hack the planet!".to_vec();
    create_response(HttpStatusCode::Ok, Some(body), true)
}

/// Handle HEAD request.
async fn handle_head(_request: &HttpMessage) -> HttpMessage {
    // Same as GET execpt don't send response with body
    let body = b"Hack the planet!".to_vec();
    create_response(HttpStatusCode::Ok, Some(body), false)
}

/// Handle POST request.
async fn handle_post(_request: &HttpMessage) -> HttpMessage {
    // Same as GET except pass request body to PHP interpreter?
    let body = b"Hack the planet!".to_vec();
    create_response(HttpStatusCode::Ok, Some(body), true)
}

/// Processes an HTTP request if able and returns a response message.
async fn process_request(request: &HttpMessage) -> HttpMessage {
    match request.header.request_line().method {
        HttpMethod::Get => handle_get(request).await,
        HttpMethod::Head => handle_head(request).await,
        HttpMethod::Post => handle_post(request).await,
    }
}

async fn read_header(stream: &mut BufReader<TcpStream>) -> Result<String, ()> {
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
                let _ = create_and_send_response(
                    stream,
                    HttpStatusCode::InternalServorError,
                    None,
                    false,
                )
                .await;
                return Err(());
            }
            Err(_) => {
                println!("Timeout, closing connection...");
                let _ =
                    create_and_send_response(stream, HttpStatusCode::RequestTimeout, None, false)
                        .await;
                return Err(());
            }
            _ => (),
        }
    }

    Ok(header)
}

async fn read_body(stream: &mut BufReader<TcpStream>, length: usize) -> Result<Vec<u8>, ()> {
    let read_timeout = Duration::from_secs(5);
    let mut body = vec![0; length];

    match timeout(read_timeout, stream.read_exact(&mut body)).await {
        Ok(Ok(0)) => {
            println!("Connection closed by client...");
            Err(())
        }
        Ok(Err(e)) => {
            eprintln!("Error reading from stream: {e}");
            let _ =
                create_and_send_response(stream, HttpStatusCode::InternalServorError, None, false)
                    .await;
            Err(())
        }
        Err(_) => {
            println!("Timeout, closing connection...");
            let _ =
                create_and_send_response(stream, HttpStatusCode::RequestTimeout, None, false).await;
            Err(())
        }
        _ => Ok(body),
    }
}

/// Handles an incoming connection from a client.
pub async fn handle_connection(stream: TcpStream, addr: SocketAddr) {
    println!("Handling connection from {addr}...");

    let mut stream = BufReader::new(stream);

    // Loop until timeout or EOF (unless keep-alive is disabled)
    'connection: loop {
        // Read and parse header
        let Ok(header) = read_header(&mut stream).await else {
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

                let _ = create_and_send_response(&mut stream, status_code, None, false).await;
                break 'connection;
            }
        };

        // Do some adjustments based on fields in header?
        if !header.is_request() {
            println!("Client sent response? Nonsense, closing connection...");
            let _ = create_and_send_response(&mut stream, HttpStatusCode::BadRequest, None, false)
                .await;
            break 'connection;
        };

        // If request contains body, read it
        // Perhaps reject if greater than some size?
        let body = if let Some(length) = header.field_lines.get("content-length") {
            let Ok(length) = length.parse() else {
                let _ =
                    create_and_send_response(&mut stream, HttpStatusCode::BadRequest, None, false)
                        .await;
                break 'connection;
            };
            match read_body(&mut stream, length).await {
                Ok(body) => body,
                Err(_) => break 'connection,
            }
        } else {
            Vec::new()
        };

        // Perform what is asked from request
        let request = HttpMessage { header, body };
        let response = process_request(&request).await;
        if send_response(&mut stream, response).await.is_err() || !request.header.is_persistent() {
            break 'connection;
        }
    }
}
