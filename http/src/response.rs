use crate::cgi::handle_php;
use crate::http::*;
use std::path::PathBuf;
use tokio::{fs::File, io::AsyncReadExt};

const SERVER_ROOT: &str = "/home/kurtjd/www";

/// Handle request.
async fn handle_request(request: &HttpMessage, send_body: bool) -> HttpMessage {
    // Check if the requested target is actually valid
    let Ok(target) = request.header.request_line().target.parse::<Target>() else {
        return create_error_response(HttpStatusCode::BadRequest).await;
    };

    let mut path = PathBuf::from(format!("{SERVER_ROOT}/public/{}", target.path));

    // Open index if path points to a folder
    if path.is_dir() {
        path.push("index.php");
    }

    // Then check if it exists on the server
    if !path.exists() {
        return create_error_response(HttpStatusCode::NotFound).await;
    }

    // Handle PHP files
    if path
        .extension()
        .and_then(|ext| ext.to_str())
        .map_or(false, |ext| ext == "php")
    {
        return if let Ok(msg) = handle_php(&path, &target.query_str, &request.body, send_body).await
        {
            msg
        } else {
            create_error_response(HttpStatusCode::InternalServorError).await
        };
    }

    // Try to open the requested file
    let Ok(mut file) = File::open(path).await else {
        return create_error_response(HttpStatusCode::InternalServorError).await;
    };

    // And finally try to read it
    let mut body = Vec::new();
    if file.read_to_end(&mut body).await.is_err() {
        return create_error_response(HttpStatusCode::InternalServorError).await;
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

/// Creates an error response with a mapped error body.
pub async fn create_error_response(status_code: HttpStatusCode) -> HttpMessage {
    let path = match status_code {
        HttpStatusCode::BadRequest => "400.html",
        HttpStatusCode::NotFound => "404.html",
        HttpStatusCode::RequestTimeout => "408.html",
        HttpStatusCode::ContentTooLarge => "413.html",
        HttpStatusCode::NotImplemented => "501.html",
        HttpStatusCode::ServiceUnavailable => "503.html",
        HttpStatusCode::HTTPVersionNotSupported => "505.html",
        _ => "500.html", // Internal servor error
    };
    let path = PathBuf::from(format!("{SERVER_ROOT}/errors/{}", path));

    // Try to open and read the error file
    let default_err = b"Unknown error occurred.".to_vec();
    let body = if let Ok(mut file) = File::open(path).await {
        let mut body = Vec::new();
        if file.read_to_end(&mut body).await.is_ok() {
            body
        } else {
            default_err
        }
    } else {
        default_err
    };

    create_response(status_code, Some(body), true)
}

/// Creates a response with server-specific fields.
pub fn create_response(
    status_code: HttpStatusCode,
    body: Option<Vec<u8>>,
    send_body: bool,
) -> HttpMessage {
    let body = body.unwrap_or_default();
    let date = chrono::Utc::now()
        .format("%a, %d %b %Y %H:%M:%S GMT")
        .to_string();

    let field_lines = [
        ("Server", "Helios/13.37"),
        ("Content-Length", &body.len().to_string()),
        ("Date", &date),
        ("Connection", "keep-alive"),
    ];

    HttpMessage::new_response(
        HttpVersion::HTTP11,
        status_code,
        &field_lines,
        if send_body { Some(body) } else { None },
    )
}
