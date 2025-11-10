use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};

use crate::{headers::headers::Headers, request::body::Body};

const MAX_LINE_LENGTH: usize = 8 * 1024;

pub struct Request {
    pub request_line: RequestLine,
    pub headers: Headers,
    pub body: Option<Body>,
}

pub struct RequestLine {
    pub http_version: String,
    pub request_target: String,
    pub method: String,
}

impl RequestLine {
    pub fn new(http_version: &str, request_target: &str, method: &str) -> Self {
        Self {
            http_version: http_version.to_string(),
            request_target: request_target.to_string(),
            method: method.to_string(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum RequestLineError {
    ReadError,
    MalformedEndOfLine,
    MalformedPart,
    MalformedMethod,
    MalformedTarget,
    BadHTTPVersion,
    LineTooLong,
}

impl std::fmt::Display for RequestLineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::ReadError => write!(f, "Error while reading bytes."),
            Self::MalformedEndOfLine => write!(f, "Malformed end of line missing '\r\n'"),
            Self::MalformedPart => {
                write!(f, "Malformed part maximum 3 parts, getting more or less")
            }
            Self::MalformedMethod => write!(f, "Malformed method, should be in uppercase only"),
            Self::MalformedTarget => write!(f, "Malformed target, should start with a slash"),
            Self::BadHTTPVersion => write!(f, "Bad http version only 1.1 supported"),
            RequestLineError::LineTooLong => write!(f, "Line too long"),
        }
    }
}

pub async fn request_from_reader(
    reader: impl AsyncRead + Unpin,
) -> Result<Request, RequestLineError> {
    let mut reader = BufReader::new(reader);

    let mut request_line_buffer = String::new();

    match reader.read_line(&mut request_line_buffer).await {
        Ok(0) => return Err(RequestLineError::ReadError),
        Ok(bytes_read) => {
            if bytes_read == MAX_LINE_LENGTH && !request_line_buffer.ends_with("\r\n") {
                return Err(RequestLineError::LineTooLong);
            }

            if !request_line_buffer.ends_with("\r\n") {
                return Err(RequestLineError::MalformedEndOfLine);
            }

            let request_line_clean = request_line_buffer.trim_end();
            let parts: Vec<&str> = request_line_clean.splitn(3, ' ').collect();

            if parts.len() != 3 {
                return Err(RequestLineError::MalformedPart);
            }

            let method = parts[0];
            let target = parts[1];
            let version = parts[2];

            if !method.chars().all(|c| c.is_ascii_uppercase()) {
                return Err(RequestLineError::MalformedMethod);
            }

            if !target.starts_with('/') {
                return Err(RequestLineError::MalformedTarget);
            }

            if version != "HTTP/1.1" {
                return Err(RequestLineError::BadHTTPVersion);
            }

            let headers = Headers::parse(&mut reader)
                .await
                .map_err(|_| RequestLineError::ReadError)?;

            let body = if let Some(content_length) = headers.headers.get("content-length") {
                Some(
                    Body::parse(&mut reader, content_length)
                        .await
                        .map_err(|_| RequestLineError::ReadError)?,
                )
            } else {
                None
            };

            Ok(Request {
                request_line: RequestLine::new("1.1", target, method),
                headers,
                body,
            })
        }
        Err(_) => Err(RequestLineError::ReadError),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn good_get_request_line() {
        match request_from_reader("GET / HTTP/1.1\r\nHost: localhost:42069\r\nUser-Agent: curl/7.81.0\r\nAccept: */*\r\n\r\n".as_bytes()).await {
            Ok(r) => {
                assert_eq!(r.request_line.method, "GET");
                assert_eq!(r.request_line.http_version, "1.1");
                assert_eq!(r.request_line.request_target, "/");
            },
            Err(e) =>panic!("{}",e),
        }
    }

    #[tokio::test]
    async fn good_get_request_line_with_path() {
        match request_from_reader("GET /coffee HTTP/1.1\r\nHost: localhost:42069\r\nUser-Agent: curl/7.81.0\r\nAccept: */*\r\n\r\n".as_bytes()).await {
            Ok(r) => {
                assert_eq!(r.request_line.method, "GET");
                assert_eq!(r.request_line.http_version, "1.1");
                assert_eq!(r.request_line.request_target, "/coffee");
            },
            Err(e) => panic!("{e}"),
        }
    }

    #[tokio::test]
    async fn invalid_version_get_request_line_with_path() {
        match request_from_reader("GET /coffee HTTP/1.3\r\nHost: localhost:42069\r\nUser-Agent: curl/7.81.0\r\nAccept: */*\r\n\r\n".as_bytes()).await {
            Ok(_) => panic!("should not pass"),
            Err(e) => assert_eq!(e, RequestLineError::BadHTTPVersion),
        }
    }

    #[tokio::test]
    async fn invalid_get_request_line_with_path() {
        match request_from_reader("GET /coffee HTTP/1.1\nHost: localhost:42069\r\nUser-Agent: curl/7.81.0\r\nAccept: */*\r\n\r\n".as_bytes()).await {
            Ok(_) => panic!("should not pass"),
            Err(e) => assert_eq!(e, RequestLineError::MalformedEndOfLine),
        }
    }

    #[tokio::test]
    async fn good_post_request_line_with_path() {
        match request_from_reader("POST /coffee HTTP/1.1\r\nHost: localhost:42069\r\nUser-Agent: curl/7.81.0\r\nAccept: */*\r\n\r\n".as_bytes()).await {
            Ok(r) => {
                assert_eq!(r.request_line.method, "POST");
                assert_eq!(r.request_line.http_version, "1.1");
                assert_eq!(r.request_line.request_target, "/coffee");
            },
            Err(e) => panic!("{e}"),
        }
    }

    #[tokio::test]
    async fn invalid_post_request_line_with_path() {
        match request_from_reader("post /coffee HTTP/1.1\r\nHost: localhost:42069\r\nUser-Agent: curl/7.81.0\r\nAccept: */*\r\n\r\n".as_bytes()).await {
            Ok(_) => panic!("should not pass"),
            Err(e) => assert_eq!(e, RequestLineError::MalformedMethod),
        }
    }

    #[tokio::test]
    async fn invalid_number_of_part_in_request_line() {
        match  request_from_reader("/coffee HTTP/1.1\r\nHost: localhost:42069\r\nUser-Agent: curl/7.81.0\r\nAccept: */*\r\n\r\n".as_bytes()).await {
            Ok(_) => panic!("should not pass"),
            Err(e) => assert_eq!(e, RequestLineError::MalformedPart)
        }
    }
}
