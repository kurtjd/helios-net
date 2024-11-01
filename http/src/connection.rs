use crate::config::Config;
use crate::http::*;
use crate::response::*;
use rustls::pki_types::pem::PemObject;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use std::future::pending;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Semaphore;
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    time::{timeout, Duration},
};
use tokio_rustls::{rustls, TlsAcceptor};
use tokio_util::either::Either;

async fn send_response(
    stream: &mut BufReader<impl AsyncWriteExt + AsyncReadExt + Unpin>,
    response: HttpMessage,
) -> std::io::Result<()> {
    stream.write_all(&Vec::from(response)).await.map_err(|e| {
        eprintln!("Error writing to stream: {e}");
        e
    })
}

async fn create_and_send_err_response(
    config: &Config,
    stream: &mut BufReader<impl AsyncWriteExt + AsyncReadExt + Unpin>,
    status_code: HttpStatusCode,
) -> std::io::Result<()> {
    let response = create_error_response(config, status_code).await;
    send_response(stream, response).await
}

async fn read_header(
    config: &Config,
    stream: &mut BufReader<impl AsyncWriteExt + AsyncReadExt + Unpin>,
) -> Result<String, ()> {
    let mut header = String::new();
    let read_timeout = Duration::from_secs(config.max_timeout);

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

async fn handle_connection(
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

async fn init_https(config: &'static Config) -> Result<(TcpListener, TlsAcceptor), ()> {
    let certs =
        match CertificateDer::pem_file_iter(format!("{}/crypt/public.pem", config.server_root)) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Error opening certificate file: {e}");
                return Err(());
            }
        };
    let certs = match certs.collect::<Result<Vec<_>, _>>() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error collecting certificates: {e}");
            return Err(());
        }
    };

    let key =
        match PrivateKeyDer::from_pem_file(format!("{}/crypt/private.pem", config.server_root)) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Error opening private key file: {e}");
                return Err(());
            }
        };
    let config_s = match rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
    {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error configuring TLS: {e}");
            return Err(());
        }
    };
    let acceptor = TlsAcceptor::from(Arc::new(config_s));
    let addr = format!("{}:{}", config.ip, config.port_https);
    let listener = match TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Error binding HTTPS to address: {e}");
            return Err(());
        }
    };
    println!("Listening on {addr} for HTTPS...");

    Ok((listener, acceptor))
}

pub async fn handle_connections(config: &'static Config, conn_sem: Arc<Semaphore>) {
    // Initialize HTTP
    let addr = format!("{}:{}", config.ip, config.port_http);
    let listener = match TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Error binding HTTP to address: {e}");
            return;
        }
    };
    println!("Listening on {addr} for HTTP...");

    // Initialize HTTPS if enabled
    let https: Option<(TcpListener, TlsAcceptor)> = if config.https_enabled {
        let Ok(s) = init_https(config).await else {
            return;
        };
        Some(s)
    } else {
        None
    };

    // Handle connections
    loop {
        tokio::select! {
            // Handle HTTP connections
            connection = listener.accept() => {
                let (stream, addr) = match connection {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("Error handling incoming HTTP connection: {e}");
                        continue;
                    }
                };
                tokio::spawn(handle_connection(
                    config,
                    stream,
                    addr,
                    Arc::clone(&conn_sem),
                ));
            }

            // Handle HTTPS connections if enabled, otherwise this future never returns
            connection = if config.https_enabled {
                let listener = &https.as_ref().expect("Will not fail since https is enabled").0;
                Either::Left(listener.accept())
            } else {
                Either::Right(pending::<std::io::Result<(TcpStream, SocketAddr)>>())
            } => {
                let (stream, addr) = match connection {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("Error handling incoming HTTPS connection: {e}");
                        continue;
                    }
                };
                let acceptor = &https.as_ref().expect("Will never get here if https is disabled").1;
                let stream = match acceptor.accept(stream).await {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("Error creating TLS stream: {e}");
                        continue;
                    }
                };
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
