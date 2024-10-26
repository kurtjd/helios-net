//! This handles CGI scripts (in our case, basically only PHP)
//!
//! This is very simplistic and does the bare basics, but
//! php-cgi is a pain in the ass and don't feel like doing more with it.
//!
//! Only supports processing form data, and not other types.

use crate::http::{HttpMessage, HttpMethod, HttpStatusCode};
use crate::response::create_response;
use std::path::Path;
use std::process::Stdio;

fn php_cgi(method: HttpMethod, path: &Path, query_str: &str) -> tokio::process::Command {
    let mut cmd = tokio::process::Command::new("php-cgi");
    cmd.env("REDIRECT_STATUS", "true")
        .env("SERVER_NAME", "Helios")
        .env("SCRIPT_FILENAME", path.to_str().unwrap())
        .env("REQUEST_METHOD", method.to_string())
        .env("QUERY_STRING", query_str);
    cmd
}

/// Process a PHP file and return the result.
pub async fn handle_php(
    path: &Path,
    query_str: &str,
    post_data: &Option<Vec<u8>>,
    send_body: bool,
) -> Result<HttpMessage, ()> {
    let cmd = if let Some(data) = post_data {
        /* We use a blocking process here because unfortunately tokio
         * does not implement piping stdout to stdin...
         *
         * And the whole reason we need this is because this is how
         * php-cgi receives POST data.
         */
        let mut echo = std::process::Command::new("echo")
            .arg(std::str::from_utf8(data).map_err(|_| ())?)
            .stdout(Stdio::piped())
            .spawn()
            .map_err(|_| ())?;

        php_cgi(HttpMethod::Post, path, query_str)
            .env("CONTENT_TYPE", "application/x-www-form-urlencoded")
            .env("CONTENT_LENGTH", data.len().to_string())
            .stdin(echo.stdout.take().ok_or(())?)
            .stdout(Stdio::piped())
            .spawn()
            .map_err(|_| ())?
    } else {
        php_cgi(HttpMethod::Get, path, query_str)
            .stdout(Stdio::piped())
            .spawn()
            .map_err(|_| ())?
    };

    let result = cmd.wait_with_output().await.map_err(|_| ())?;
    if result.status.success() {
        let (_header, body) = std::str::from_utf8(&result.stdout)
            .map_err(|_| ())?
            .split_once("\r\n\r\n")
            .ok_or(())?;

        let body = body.as_bytes().to_vec();
        Ok(create_response(HttpStatusCode::Ok, Some(body), send_body))
    } else {
        Err(())
    }
}
