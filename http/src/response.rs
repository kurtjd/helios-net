use crate::cgi::handle_php;
use crate::http::*;
use std::path::PathBuf;
use tokio::{fs::File, io::AsyncReadExt};

const SERVER_ROOT: &str = "/home/kurtjd/webserver";

/// Handle request.
async fn handle_request(request: &HttpMessage, send_body: bool) -> HttpMessage {
    // Check if the requested target is actually valid
    let Ok(target) = request.header.request_line().target.parse::<Target>() else {
        return create_response(HttpStatusCode::BadRequest, None, false);
    };

    let mut path = PathBuf::from(format!("{SERVER_ROOT}/{}", target.path));

    // Open index if path points to a folder
    if path.is_dir() {
        path.push("index.php");
    }

    // Then check if it exists on the server
    if !path.exists() {
        let body = b"404 Not Found".to_vec();
        return create_response(HttpStatusCode::NotFound, Some(body), send_body);
    }

    // Handle PHP files
    if path
        .extension()
        .and_then(|ext| ext.to_str())
        .map_or(false, |ext| ext == "php")
    {
        return handle_php(&path, &target.query_str, &request.body, send_body).await;
    }

    // Try to open the requested file
    let Ok(mut file) = File::open(path).await else {
        return create_response(HttpStatusCode::InternalServorError, None, false);
    };

    // And finally try to read it
    let mut body = Vec::new();
    if file.read_to_end(&mut body).await.is_err() {
        return create_response(HttpStatusCode::InternalServorError, None, false);
    }

    create_response(HttpStatusCode::Ok, Some(body), send_body)
}

/// Processes an HTTP request if able and returns a response message.
pub async fn process_request(request: &HttpMessage) -> HttpMessage {
    match request.header.request_line().method {
        HttpMethod::Get | HttpMethod::Post => handle_request(request, true).await,
        HttpMethod::Head => handle_request(request, false).await,
    }
}

/// Creates a response with server-specific fields.
pub fn create_response(
    status_code: HttpStatusCode,
    body: Option<Vec<u8>>,
    send_body: bool,
) -> HttpMessage {
    let body = body.unwrap_or_default();

    // TODO: Make these more dynamic and dependent on particular request
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
        if send_body { Some(body) } else { None },
    )
}
