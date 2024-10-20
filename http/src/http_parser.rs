use std::collections::HashMap;
use std::fmt::{Display, Write};
use std::str::FromStr;

#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HttpVersion {
    HTTP09,
    HTTP10,
    HTTP11,
    HTTP2,
    HTTP3,
}

impl Display for HttpVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::HTTP09 => write!(f, "HTTP/0.9"),
            Self::HTTP10 => write!(f, "HTTP/1.0"),
            Self::HTTP11 => write!(f, "HTTP/1.1"),
            Self::HTTP2 => write!(f, "HTTP/2"),
            Self::HTTP3 => write!(f, "HTTP/3"),
        }
    }
}

impl TryFrom<&str> for HttpVersion {
    type Error = ();

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "HTTP/0.9" => Ok(Self::HTTP09),
            "HTTP/1.0" => Ok(Self::HTTP10),
            "HTTP/1.1" => Ok(Self::HTTP11),
            "HTTP/2" => Ok(Self::HTTP2),
            "HTTP/3" => Ok(Self::HTTP3),
            _ => Err(()),
        }
    }
}

#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HttpStatusCode {
    Continue,
    SwitchingProtocols,
    Ok,
    Created,
    Accepted,
    NonAuthoritativeInformation,
    NoContent,
    ResetContent,
    PartialContent,
    MultipleChoices,
    MovedPermanently,
    Found,
    SeeOther,
    NotModified,
    UseProxy,
    TemporaryRedirect,
    PermanentRedirect,
    BadRequest,
    Unauthorized,
    PaymentRequired,
    Forbidden,
    NotFound,
    MethodNotAllowed,
    NotAcceptable,
    ProxyAuthenticationRequired,
    RequestTimeout,
    Conflict,
    Gone,
    LengthRequired,
    PreconditionFailed,
    ContentToolarge,
    URITooLong,
    UnsupportedMediaType,
    RangeNotSatisfiable,
    ExpectationFailed,
    MisdirectedRequest,
    UnprocessableContent,
    UpgradeRequired,
    InternalServorError,
    NotImplemented,
    BadGateway,
    ServiceUnavailable,
    GatewayTimeout,
    HTTPVersionNotSupported,
}

impl Display for HttpStatusCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Continue => write!(f, "Continue"),
            Self::SwitchingProtocols => write!(f, "Switching Protocols"),
            Self::Ok => write!(f, "OK"),
            Self::Created => write!(f, "Created"),
            Self::Accepted => write!(f, "Accepted"),
            Self::NonAuthoritativeInformation => write!(f, "Non-Authoritative Information"),
            Self::NoContent => write!(f, "No Content"),
            Self::ResetContent => write!(f, "Reset Content"),
            Self::PartialContent => write!(f, "Partial Content"),
            Self::MultipleChoices => write!(f, "Multiple Choices"),
            Self::MovedPermanently => write!(f, "Moved Permanently"),
            Self::Found => write!(f, "Found"),
            Self::SeeOther => write!(f, "See Other"),
            Self::NotModified => write!(f, "Not Modified"),
            Self::UseProxy => write!(f, "Use Proxy"),
            Self::TemporaryRedirect => write!(f, "Temporary Redirect"),
            Self::PermanentRedirect => write!(f, "Permanent Redirect"),
            Self::BadRequest => write!(f, "Bad Request"),
            Self::Unauthorized => write!(f, "Unauthorized"),
            Self::PaymentRequired => write!(f, "Payment Required"),
            Self::Forbidden => write!(f, "Forbidden"),
            Self::NotFound => write!(f, "Not Found"),
            Self::MethodNotAllowed => write!(f, "Method Not Allowed"),
            Self::NotAcceptable => write!(f, "Not Acceptable"),
            Self::ProxyAuthenticationRequired => write!(f, "Proxy Authentication Required"),
            Self::RequestTimeout => write!(f, "Request Timeout"),
            Self::Conflict => write!(f, "Conflict"),
            Self::Gone => write!(f, "Gone"),
            Self::LengthRequired => write!(f, "Length Required"),
            Self::PreconditionFailed => write!(f, "Precondition Failed"),
            Self::ContentToolarge => write!(f, "Content Too Large"),
            Self::URITooLong => write!(f, "URI Too Long"),
            Self::UnsupportedMediaType => write!(f, "Unsupported Media Type"),
            Self::RangeNotSatisfiable => write!(f, "RangeNotSatisfiable"),
            Self::ExpectationFailed => write!(f, "Expectation Failed"),
            Self::MisdirectedRequest => write!(f, "Misdirected Request"),
            Self::UnprocessableContent => write!(f, "Unprocessable Content"),
            Self::UpgradeRequired => write!(f, "Upgrade Required"),
            Self::InternalServorError => write!(f, "Internal Servor Error"),
            Self::NotImplemented => write!(f, "NotImplemented"),
            Self::BadGateway => write!(f, "Bad Gateway"),
            Self::ServiceUnavailable => write!(f, "Service Unavailable"),
            Self::GatewayTimeout => write!(f, "Gateway Timeout"),
            Self::HTTPVersionNotSupported => write!(f, "HTTP Version Not Supported"),
        }
    }
}

impl From<HttpStatusCode> for u16 {
    fn from(status_code: HttpStatusCode) -> Self {
        match status_code {
            HttpStatusCode::Continue => 100,
            HttpStatusCode::SwitchingProtocols => 101,
            HttpStatusCode::Ok => 200,
            HttpStatusCode::Created => 201,
            HttpStatusCode::Accepted => 202,
            HttpStatusCode::NonAuthoritativeInformation => 203,
            HttpStatusCode::NoContent => 204,
            HttpStatusCode::ResetContent => 205,
            HttpStatusCode::PartialContent => 206,
            HttpStatusCode::MultipleChoices => 300,
            HttpStatusCode::MovedPermanently => 301,
            HttpStatusCode::Found => 302,
            HttpStatusCode::SeeOther => 303,
            HttpStatusCode::NotModified => 304,
            HttpStatusCode::UseProxy => 305,
            HttpStatusCode::TemporaryRedirect => 307,
            HttpStatusCode::PermanentRedirect => 308,
            HttpStatusCode::BadRequest => 400,
            HttpStatusCode::Unauthorized => 401,
            HttpStatusCode::PaymentRequired => 402,
            HttpStatusCode::Forbidden => 403,
            HttpStatusCode::NotFound => 404,
            HttpStatusCode::MethodNotAllowed => 405,
            HttpStatusCode::NotAcceptable => 406,
            HttpStatusCode::ProxyAuthenticationRequired => 407,
            HttpStatusCode::RequestTimeout => 408,
            HttpStatusCode::Conflict => 409,
            HttpStatusCode::Gone => 410,
            HttpStatusCode::LengthRequired => 411,
            HttpStatusCode::PreconditionFailed => 412,
            HttpStatusCode::ContentToolarge => 413,
            HttpStatusCode::URITooLong => 414,
            HttpStatusCode::UnsupportedMediaType => 415,
            HttpStatusCode::RangeNotSatisfiable => 416,
            HttpStatusCode::ExpectationFailed => 417,
            HttpStatusCode::MisdirectedRequest => 421,
            HttpStatusCode::UnprocessableContent => 422,
            HttpStatusCode::UpgradeRequired => 426,
            HttpStatusCode::InternalServorError => 500,
            HttpStatusCode::NotImplemented => 501,
            HttpStatusCode::BadGateway => 502,
            HttpStatusCode::ServiceUnavailable => 503,
            HttpStatusCode::GatewayTimeout => 504,
            HttpStatusCode::HTTPVersionNotSupported => 505,
        }
    }
}

impl TryFrom<u16> for HttpStatusCode {
    type Error = ();

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            200 => Ok(Self::Ok),
            404 => Ok(Self::NotFound),
            /* Implement more as needed. */
            _ => Err(()),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HttpMethod {
    Get,
    Head,
    Post,
    Put,
    Delete,
    Connect,
    Options,
    Trace,
}

impl Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Get => write!(f, "GET"),
            Self::Head => write!(f, "HEAD"),
            Self::Post => write!(f, "POST"),
            Self::Put => write!(f, "PUT"),
            Self::Delete => write!(f, "DELETE"),
            Self::Connect => write!(f, "CONNECT"),
            Self::Options => write!(f, "OPTIONS"),
            Self::Trace => write!(f, "TRACE"),
        }
    }
}

impl TryFrom<&str> for HttpMethod {
    type Error = ();

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "GET" => Ok(Self::Get),
            "HEAD" => Ok(Self::Head),
            "POST" => Ok(Self::Post),
            "PUT" => Ok(Self::Put),
            "DELETE" => Ok(Self::Delete),
            "CONNECT" => Ok(Self::Connect),
            "OPTIONS" => Ok(Self::Options),
            "TRACE" => Ok(Self::Trace),
            _ => Err(()),
        }
    }
}

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

pub struct HttpStatusLine {
    http_version: HttpVersion,
    status_code: HttpStatusCode,
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
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut tokens = s.split(' ');

        let http_version: HttpVersion = tokens.next().ok_or(())?.try_into()?;
        let status_code: HttpStatusCode = tokens
            .next()
            .ok_or(())?
            .parse::<u16>()
            .map_err(|_| ())?
            .try_into()?;

        /* Reason phrase is optional, and we don't really care about it,
         * but there must be a space after the status code, thus
         * this token must exist even if it's empty. Otherwise
         * the response is malformed.
         */
        let _reason_phrase = tokens.next().ok_or(())?;

        // If we still have remaining tokens the response is malformed
        if tokens.next().is_none() {
            Ok(Self {
                http_version,
                status_code,
            })
        } else {
            Err(())
        }
    }
}

pub struct HttpRequestLine {
    method: HttpMethod,
    target: String,
    http_version: HttpVersion,
}

impl Display for HttpRequestLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {}", self.method, self.target, self.http_version)
    }
}

impl FromStr for HttpRequestLine {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut tokens = s.split(' ');

        let method: HttpMethod = tokens.next().ok_or(())?.try_into()?;
        let target = String::from(tokens.next().ok_or(())?);
        let http_version: HttpVersion = tokens.next().ok_or(())?.try_into()?;

        // If we still have remaining tokens the request is malformed
        if tokens.next().is_none() {
            Ok(Self {
                method,
                target,
                http_version,
            })
        } else {
            Err(())
        }
    }
}

pub struct HttpField {
    name: String,
    value: String,
}

impl Display for HttpField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.name, self.value)
    }
}

impl FromStr for HttpField {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (name, value) = s.split_once(':').ok_or(())?;

        if name.chars().any(char::is_whitespace) {
            Err(())
        } else {
            Ok(HttpField {
                name: String::from(name),
                value: String::from(value.trim()),
            })
        }
    }
}

pub struct HttpHeader {
    start_line: HttpStartLine,
    field_lines: HashMap<String, String>,
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
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        /* Optional whitespace around field values might cause empty lines,
         * which are valid but shouldn't be processed.
         */
        let mut lines = s.split("\r\n").filter(|s| !s.is_empty());

        /* Quick way to figure out if we are dealing with a response or request,
         * as responses always start with HTTP version string (which of course
         * starts with "HTTP").
         */
        let start_line = lines.next().ok_or(())?;

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
            .collect::<Result<HashMap<String, String>, ()>>()?;

        Ok(HttpHeader {
            start_line,
            field_lines,
        })
    }
}

pub struct HttpMessage {
    header: HttpHeader,
    body: Vec<u8>,
}

impl From<HttpMessage> for Vec<u8> {
    fn from(message: HttpMessage) -> Self {
        message
            .header
            .to_string()
            .into_bytes()
            .into_iter()
            .chain(message.body)
            .collect()
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
}
