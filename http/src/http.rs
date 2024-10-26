use percent_encoding::percent_decode_str;
use std::collections::HashMap;
use std::fmt::{Display, Write};
use std::str::FromStr;
use url::Url;

#[non_exhaustive]
#[derive(Clone, Copy, Debug)]
pub enum Error {
    Malformed,
    UnsupportedMethod,
    UnsupportedVersion,
    UnsupportedStatusCode,
}

#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HttpVersion {
    HTTP10,
    HTTP11,
}

impl Display for HttpVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::HTTP10 => write!(f, "HTTP/1.0"),
            Self::HTTP11 => write!(f, "HTTP/1.1"),
        }
    }
}

impl TryFrom<&str> for HttpVersion {
    type Error = Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "HTTP/1.0" => Ok(Self::HTTP10),
            "HTTP/1.1" => Ok(Self::HTTP11),
            _ => Err(Error::UnsupportedVersion),
        }
    }
}

#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HttpStatusCode {
    Ok,
    BadRequest,
    NotFound,
    RequestTimeout,
    ContentTooLarge,
    InternalServorError,
    NotImplemented,
    ServiceUnavailable,
    HTTPVersionNotSupported,
}

impl Display for HttpStatusCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ok => write!(f, "OK"),
            Self::BadRequest => write!(f, "Bad Request"),
            Self::NotFound => write!(f, "Not Found"),
            Self::RequestTimeout => write!(f, "Request Timeout"),
            Self::ContentTooLarge => write!(f, "Content Too Large"),
            Self::InternalServorError => write!(f, "Internal Servor Error"),
            Self::NotImplemented => write!(f, "Not Implemented"),
            Self::ServiceUnavailable => write!(f, "Service Unavailable"),
            Self::HTTPVersionNotSupported => write!(f, "HTTP Version Not Supported"),
        }
    }
}

impl From<HttpStatusCode> for u16 {
    fn from(status_code: HttpStatusCode) -> Self {
        match status_code {
            HttpStatusCode::Ok => 200,
            HttpStatusCode::BadRequest => 400,
            HttpStatusCode::NotFound => 404,
            HttpStatusCode::RequestTimeout => 408,
            HttpStatusCode::ContentTooLarge => 413,
            HttpStatusCode::InternalServorError => 500,
            HttpStatusCode::NotImplemented => 501,
            HttpStatusCode::ServiceUnavailable => 503,
            HttpStatusCode::HTTPVersionNotSupported => 505,
        }
    }
}

impl TryFrom<u16> for HttpStatusCode {
    type Error = Error;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            200 => Ok(Self::Ok),
            400 => Ok(Self::BadRequest),
            404 => Ok(Self::NotFound),
            408 => Ok(Self::RequestTimeout),
            413 => Ok(Self::ContentTooLarge),
            500 => Ok(Self::InternalServorError),
            501 => Ok(Self::NotImplemented),
            505 => Ok(Self::HTTPVersionNotSupported),
            _ => Err(Error::UnsupportedStatusCode),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HttpMethod {
    Get,
    Head,
    Post,
}

impl Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Get => write!(f, "GET"),
            Self::Head => write!(f, "HEAD"),
            Self::Post => write!(f, "POST"),
        }
    }
}

impl TryFrom<&str> for HttpMethod {
    type Error = Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "GET" => Ok(Self::Get),
            "HEAD" => Ok(Self::Head),
            "POST" => Ok(Self::Post),
            _ => Err(Error::UnsupportedMethod),
        }
    }
}

#[derive(Debug)]
pub enum HttpStartLine {
    Response(HttpStatusLine),
    Request(HttpRequestLine),
}

impl Display for HttpStartLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpStartLine::Response(resp) => write!(f, "{resp}"),
            HttpStartLine::Request(req) => write!(f, "{req}"),
        }
    }
}

#[derive(Debug)]
pub struct HttpStatusLine {
    pub http_version: HttpVersion,
    pub status_code: HttpStatusCode,
}

impl Display for HttpStatusLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} {}",
            self.http_version,
            u16::from(self.status_code),
            self.status_code
        )
    }
}

impl FromStr for HttpStatusLine {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut tokens = s.split(' ');

        let http_version: HttpVersion = tokens.next().ok_or(Error::Malformed)?.try_into()?;
        let status_code: HttpStatusCode = tokens
            .next()
            .ok_or(Error::Malformed)?
            .parse::<u16>()
            .map_err(|_| Error::Malformed)?
            .try_into()?;

        /* Reason phrase is optional, and we don't really care about it,
         * but there must be a space after the status code, thus
         * this token must exist even if it's empty. Otherwise
         * the response is malformed.
         */
        let _reason_phrase = tokens.next().ok_or(Error::Malformed)?;

        // If we still have remaining tokens the response is malformed
        if tokens.next().is_none() {
            Ok(Self {
                http_version,
                status_code,
            })
        } else {
            Err(Error::Malformed)
        }
    }
}

#[derive(Debug)]
pub struct HttpRequestLine {
    pub method: HttpMethod,
    pub target: String,
    pub http_version: HttpVersion,
}

impl Display for HttpRequestLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {}", self.method, self.target, self.http_version)
    }
}

impl FromStr for HttpRequestLine {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut tokens = s.split(' ');

        let method: HttpMethod = tokens.next().ok_or(Error::Malformed)?.try_into()?;
        let target = String::from(tokens.next().ok_or(Error::Malformed)?);
        let http_version: HttpVersion = tokens.next().ok_or(Error::Malformed)?.try_into()?;

        // If we still have remaining tokens the request is malformed
        if tokens.next().is_none() {
            Ok(Self {
                method,
                target,
                http_version,
            })
        } else {
            Err(Error::Malformed)
        }
    }
}

pub struct HttpField {
    pub name: String,
    pub value: String,
}

impl Display for HttpField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.name, self.value)
    }
}

impl FromStr for HttpField {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (name, value) = s.split_once(':').ok_or(Error::Malformed)?;

        if name.chars().any(char::is_whitespace) {
            Err(Error::Malformed)
        } else {
            Ok(HttpField {
                name: String::from(name),
                value: String::from(value.trim()),
            })
        }
    }
}

#[derive(Debug)]
pub struct HttpHeader {
    pub start_line: HttpStartLine,
    pub field_lines: HashMap<String, String>,
}

impl HttpHeader {
    /// Returns true if header requests a persistent connection, false otherwise.
    pub fn is_persistent(&self) -> bool {
        let version = if let HttpStartLine::Request(req) = &self.start_line {
            req.http_version
        } else {
            return false;
        };

        match version {
            // HTTP/1.1 is persistent by default
            HttpVersion::HTTP11 => self
                .field_lines
                .get("connection")
                .map_or(true, |v| v == "keep-alive"),

            // HTTP/1.0 is NOT persistent by default
            HttpVersion::HTTP10 => self
                .field_lines
                .get("connection")
                .map_or(false, |v| v == "keep-alive"),
        }
    }

    /// Returns true is header represents a request, false otherwise.
    pub fn is_request(&self) -> bool {
        matches!(&self.start_line, HttpStartLine::Request(_))
    }

    /// Returns a reference to the request line if request header,
    /// panics otherwise.
    pub fn request_line(&self) -> &HttpRequestLine {
        if let HttpStartLine::Request(request_line) = &self.start_line {
            request_line
        } else {
            panic!("Header is not an HTTP request.");
        }
    }
}

impl Display for HttpHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let start_line = format!("{}\r\n", self.start_line);
        let fields: String =
            self.field_lines
                .iter()
                .fold(String::new(), |mut output, (field_name, field_value)| {
                    let _ = write!(output, "{field_name}: {field_value}\r\n");
                    output
                });

        write!(f, "{start_line}{fields}\r\n")
    }
}

impl FromStr for HttpHeader {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        /* Optional whitespace around field values might cause empty lines,
         * which are valid but shouldn't be processed.
         */
        let mut lines = s.split("\r\n").filter(|s| !s.is_empty());

        /* Quick way to figure out if we are dealing with a response or request,
         * as responses always start with HTTP version string (which of course
         * starts with "HTTP").
         */
        let start_line = lines.next().ok_or(Error::Malformed)?;

        let start_line = if start_line.starts_with("HTTP") {
            HttpStartLine::Response(start_line.parse()?)
        } else {
            HttpStartLine::Request(start_line.parse()?)
        };

        // Collect all fields (remaining lines) into a hashmap as field-name/field-value pairs
        let field_lines = lines
            .map(|line| {
                line.parse::<HttpField>()
                    .map(|field| (field.name.to_lowercase(), field.value))
            })
            .collect::<Result<HashMap<String, String>, Error>>()?;

        Ok(HttpHeader {
            start_line,
            field_lines,
        })
    }
}

pub struct HttpMessage {
    pub header: HttpHeader,
    pub body: Option<Vec<u8>>,
}

impl HttpMessage {
    pub fn new_response(
        http_version: HttpVersion,
        status_code: HttpStatusCode,
        field_lines: &[(&str, &str)],
        body: Option<Vec<u8>>,
    ) -> Self {
        let start_line = HttpStartLine::Response(HttpStatusLine {
            http_version,
            status_code,
        });

        let field_lines = field_lines
            .iter()
            .copied()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        let header = HttpHeader {
            start_line,
            field_lines,
        };
        Self { header, body }
    }
}

impl From<HttpMessage> for Vec<u8> {
    fn from(message: HttpMessage) -> Self {
        message
            .header
            .to_string()
            .into_bytes()
            .into_iter()
            .chain(message.body.unwrap_or_default())
            .collect()
    }
}

pub struct Target {
    pub path: String,
    pub query_str: String,
}

impl FromStr for Target {
    type Err = ();

    // TODO: Write unit tests
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        /* We don't really care about the URL, but the URL crate wants
         * a full, valid URL for us to use it's useful bits.
         * Hence why we append the target to something arbitrary like
         * http://localhost
         */
        let url = format!("http://localhost/{}", s.trim_start_matches('/'));
        let url = Url::parse(&url).map_err(|_| ())?;

        let path = percent_decode_str(url.path())
            .decode_utf8()
            .map_err(|_| ())?
            .into_owned()
            .trim_start_matches('/')
            .to_string();

        let query_str = url.query().unwrap_or("").to_string();

        Ok(Self { path, query_str })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_line_from_str() {
        // Valid (with optional description)
        let status_line: HttpStatusLine = "HTTP/1.1 200 OK".parse().unwrap();
        assert_eq!(status_line.http_version, HttpVersion::HTTP11);
        assert_eq!(status_line.status_code, HttpStatusCode::Ok);

        // Valid (without optional description)
        let status_line: HttpStatusLine = "HTTP/1.1 200 ".parse().unwrap();
        assert_eq!(status_line.http_version, HttpVersion::HTTP11);
        assert_eq!(status_line.status_code, HttpStatusCode::Ok);

        // Invalid (invalid response code)
        assert!("HTTP/1.1 1337 Wtf".parse::<HttpStatusLine>().is_err());

        // Invalid (invalid HTTP version)
        assert!("HTTP/4.2 200 OK".parse::<HttpStatusLine>().is_err());

        // Invalid (missing trailing space)
        assert!("HTTP/1.1 200".parse::<HttpStatusLine>().is_err());

        // Invalid (multiple spaces between tokens)
        assert!("HTTP/1.1  200 OK".parse::<HttpStatusLine>().is_err());

        // Invalid (extra trailing tokens)
        assert!("HTTP/1.1 200 OK WTF".parse::<HttpStatusLine>().is_err());

        // Invalid (malformed)
        assert!("".parse::<HttpStatusLine>().is_err());

        // Invalid (malformed)
        assert!("\r\n".parse::<HttpStatusLine>().is_err());

        // Invalid (malformed)
        assert!("Hack the planet!".parse::<HttpStatusLine>().is_err());
    }

    #[test]
    fn test_status_line_to_str() {
        let status_line = HttpStatusLine {
            http_version: HttpVersion::HTTP11,
            status_code: HttpStatusCode::Ok,
        };

        assert_eq!(status_line.to_string(), "HTTP/1.1 200 OK");
    }

    #[test]
    fn test_request_line_from_str() {
        // Valid
        let request_line: HttpRequestLine = "GET /index.html HTTP/1.1".parse().unwrap();
        assert_eq!(request_line.method, HttpMethod::Get);
        assert_eq!(request_line.target, String::from("/index.html"));
        assert_eq!(request_line.http_version, HttpVersion::HTTP11);

        // Invalid (invalid method)
        assert!("FOO / HTTP/1.1".parse::<HttpRequestLine>().is_err());

        // Invalid (invalid HTTP version)
        assert!("GET / HTTP/4.2".parse::<HttpRequestLine>().is_err());

        // Invalid (target contains whitespace)
        assert!("GET /guest book.html HTTP/4.2"
            .parse::<HttpRequestLine>()
            .is_err());

        // Invalid (multiple spaces between tokens)
        assert!("GET  / HTTP/1.1".parse::<HttpRequestLine>().is_err());

        // Invalid (extra trailing tokens)
        assert!("GET / HTTP/1.1 WTF".parse::<HttpRequestLine>().is_err());

        // Invalid (malformed)
        assert!("\r\n".parse::<HttpRequestLine>().is_err());

        // Invalid (malformed)
        assert!("Hack the planet!".parse::<HttpRequestLine>().is_err());
    }

    #[test]
    fn test_request_line_to_str() {
        let request_line = HttpRequestLine {
            method: HttpMethod::Get,
            target: String::from("/index.html"),
            http_version: HttpVersion::HTTP11,
        };

        assert_eq!(request_line.to_string(), "GET /index.html HTTP/1.1")
    }

    #[test]
    fn test_field_line_from_str() {
        // Valid
        let field_line: HttpField = "Connection:keep-alive".parse().unwrap();
        assert_eq!(field_line.name, "Connection");
        assert_eq!(field_line.value, "keep-alive");

        // Valid (field value contains colon)
        let field_line: HttpField = "Host:localhost:42".parse().unwrap();
        assert_eq!(field_line.name, "Host");
        assert_eq!(field_line.value, "localhost:42");

        // Valid (optional whitespace)
        let field_line: HttpField = "Connection: keep-alive".parse().unwrap();
        assert_eq!(field_line.name, "Connection");
        assert_eq!(field_line.value, "keep-alive");

        // Valid (optional whitespace)
        let field_line: HttpField = "Connection:keep-alive ".parse().unwrap();
        assert_eq!(field_line.name, "Connection");
        assert_eq!(field_line.value, "keep-alive");

        // Valid (optional whitespace)
        let field_line: HttpField = "Connection: keep-alive ".parse().unwrap();
        assert_eq!(field_line.name, "Connection");
        assert_eq!(field_line.value, "keep-alive");

        // Invalid (whitespace in field name)
        assert!("Connection : keep-alive".parse::<HttpField>().is_err());
    }

    #[test]
    fn test_field_line_to_str() {
        let field_line = HttpField {
            name: String::from("Connection"),
            value: String::from("keep-alive"),
        };

        assert_eq!(field_line.to_string(), "Connection: keep-alive");
    }

    #[test]
    fn test_header_from_str_valid() {
        // Valid (Request with no field lines)
        let header: HttpHeader = "GET /index.html HTTP/1.1\r\n".parse().unwrap();
        if let HttpStartLine::Request(req) = &header.start_line {
            assert_eq!(req.method, HttpMethod::Get);
            assert_eq!(req.target, String::from("/index.html"));
            assert_eq!(req.http_version, HttpVersion::HTTP11);
        } else {
            panic!("Expected Request");
        }

        // Valid (Request with field lines)
        let header: HttpHeader =
            "GET /index.html HTTP/1.1\r\nConnection: keep-alive\r\nHost: localhost:42\r\n"
                .parse()
                .unwrap();
        if let HttpStartLine::Request(req) = &header.start_line {
            assert_eq!(req.method, HttpMethod::Get);
            assert_eq!(req.target, String::from("/index.html"));
            assert_eq!(req.http_version, HttpVersion::HTTP11);
        } else {
            panic!("Expected Request");
        }
        assert_eq!(header.field_lines["connection"], "keep-alive");
        assert_eq!(header.field_lines["host"], "localhost:42");

        // Valid (Response with no field lines)
        let header: HttpHeader = "HTTP/1.1 200 OK\r\n".parse().unwrap();
        if let HttpStartLine::Response(resp) = &header.start_line {
            assert_eq!(resp.http_version, HttpVersion::HTTP11);
            assert_eq!(resp.status_code, HttpStatusCode::Ok);
        } else {
            panic!("Expected Response");
        }

        // Valid (Response with field lines)
        let header: HttpHeader =
            "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: 1337\r\n"
                .parse()
                .unwrap();
        if let HttpStartLine::Response(resp) = &header.start_line {
            assert_eq!(resp.http_version, HttpVersion::HTTP11);
            assert_eq!(resp.status_code, HttpStatusCode::Ok);
        } else {
            panic!("Expected Response");
        }

        assert_eq!(header.field_lines["content-type"], "text/html");
        assert_eq!(header.field_lines["content-length"], "1337");
    }

    #[test]
    fn test_header_from_str_invalid() {
        // Invalid (Bad status code)
        assert!("HTTP/1.1 777 Wtf\r\nContent-Type: text/html\r\n"
            .parse::<HttpHeader>()
            .is_err());
    }

    #[test]
    fn test_header_to_str() {
        let start_line = HttpStartLine::Request("GET /index.html HTTP/1.1".parse().unwrap());
        let field_lines = HashMap::from([
            (String::from("connection"), String::from("keep-alive")),
            (String::from("host"), String::from("localhost:42")),
        ]);

        let _header = HttpHeader {
            start_line,
            field_lines,
        };

        // Difficult as the order elements are stored in hashmap is non-deterministic...
        /*assert_eq!(
            header.to_string(),
            "GET /index.html HTTP/1.1\r\nconnection: keep-alive\r\nhost: localhost:42\r\n\r\n"
        );*/
    }

    #[test]
    fn test_target_from_str() {
        // Simple test
        let target: Target = "/index.php".parse().unwrap();
        assert_eq!(target.path, "index.php");
        assert_eq!(target.query_str, "");

        // No leading backslash test
        let target: Target = "index.php".parse().unwrap();
        assert_eq!(target.path, "index.php");
        assert_eq!(target.query_str, "");

        // Query string test
        let target: Target = "/index.php?msg=Hack the planet!&foo=bar".parse().unwrap();
        assert_eq!(target.path, "index.php");
        assert_eq!(target.query_str, "msg=Hack%20the%20planet!&foo=bar");

        // Thwart hacker test
        let target: Target = "../../../secrets.lol".parse().unwrap();
        assert_eq!(target.path, "secrets.lol");
        assert_eq!(target.query_str, "");

        // Thwart weird hacker test
        let target: Target = "/some/folder/../../../secrets.lol".parse().unwrap();
        assert_eq!(target.path, "secrets.lol");
        assert_eq!(target.query_str, "");
    }
}
