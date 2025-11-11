use chrono::Local;
use flate2::{Compression, write::GzEncoder};
use serde_json::Value;
use std::io::Write;

use crate::{headers::Headers, request::Body};

#[derive(Clone)]
pub struct Response {
    response_line: ResponseLine,
    content: Vec<u8>,
    headers: Headers,
    body: Option<Body>,
}

impl Response {
    pub fn new() -> Self {
        let mut response = Self {
            content: Vec::new(),
            headers: Headers::new(),
            response_line: ResponseLine {
                version: Version::OneDotOne,
                status: Status::Ok,
            },
            body: None,
        };
        response.set_header("Server", "rust");
        response
    }

    pub fn set_header(&mut self, key: &str, value: &str) {
        self.headers.set(key, value).unwrap();
    }

    pub fn status(&mut self, status: Status) -> Self {
        self.response_line.status = status;
        self.clone()
    }

    pub fn body(&mut self, body: Vec<u8>) {
        let content_type = if serde_json::from_slice::<Value>(&body).is_ok() {
            "application/json"
        } else {
            if let Some(kind) = infer::get(&body) {
                kind.mime_type()
            } else {
                "text/plain; charset=utf-8"
            }
        };
        self.set_header("Content-type", content_type);
        self.body = Some(Body::new(body));
    }

    pub fn send(&mut self, accept_encoding: Option<&String>) -> Vec<u8> {
        self.content.append(&mut Vec::from(
            format!("{}", &mut self.response_line.version).as_bytes(),
        ));
        self.content.append(&mut Vec::from(
            format!("{}", &mut self.response_line.status).as_bytes(),
        ));
        let now = Local::now();
        self.auto_compress(accept_encoding).unwrap();
        self.headers.set("Date", now.to_rfc2822().as_str()).unwrap();
        self.content.append(&mut self.headers.to_bytes());
        if let Some(body) = &self.body {
            self.content.append(&mut Vec::from(body.as_bytes()));
        }
        self.content.clone()
    }

    fn compress_gzip(&mut self) -> Result<(), std::io::Error> {
        if let Some(body) = &mut self.body {
            let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(body.as_bytes())?;
            let compressed = encoder.finish()?;
            self.headers.set("Content-Encoding", "gzip").unwrap();
            self.headers
                .set("Content-length", format!("{}", &compressed.len()).as_str())
                .unwrap();
            body.set(compressed);
        }
        Ok(())
    }

    pub fn should_compress(&self) -> bool {
        if let Some(content_type) = self.headers.get("content-type") {
            matches!(
                content_type.as_str(),
                "text/html"
                    | "text/css"
                    | "application/javascript"
                    | "application/json"
                    | "text/plain"
            )
        } else {
            false
        }
    }

    pub fn auto_compress(
        &mut self,
        accept_encoding: Option<&String>,
    ) -> Result<(), std::io::Error> {
        if let Some(encoding) = accept_encoding {
            if encoding.contains("gzip") && self.should_compress() {
                self.compress_gzip()?;
            }
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct ResponseLine {
    version: Version,
    status: Status,
}

#[derive(Clone)]
pub enum Version {
    OneDotOne,
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::OneDotOne => write!(f, "HTTP/1.1 "),
        }
    }
}

#[derive(Clone)]
pub enum Status {
    // 2xx Success
    Ok,
    Created,
    Accepted,
    NoContent,
    PartialContent,

    // 3xx Redirection
    MovedPermanently,
    Found,
    NotModified,

    // 4xx Client Errors
    BadRequest,
    Unauthorized,
    Forbidden,
    NotFound,
    MethodNotAllowed,
    Conflict,
    UnprocessableContent,
    TooManyRequests,

    // 5xx Server Errors
    InternalServerError,
    NotImplemented,
    ServiceUnavailable,
    GatewayTimeout,
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            // 2xx
            Self::Ok => write!(f, "200 OK\r\n"),
            Self::Created => write!(f, "201 Created\r\n"),
            Self::Accepted => write!(f, "202 Accepted\r\n"),
            Self::NoContent => write!(f, "204 No Content\r\n"),
            Self::PartialContent => write!(f, "206 Partial Content"),

            // 3xx
            Self::MovedPermanently => write!(f, "301 Moved Permanently\r\n"),
            Self::Found => write!(f, "302 Found\r\n"),
            Self::NotModified => write!(f, "304 Not Modified\r\n"),

            // 4xx
            Self::BadRequest => write!(f, "400 Bad Request\r\n"),
            Self::Unauthorized => write!(f, "401 Unauthorized\r\n"),
            Self::Forbidden => write!(f, "403 Forbidden\r\n"),
            Self::NotFound => write!(f, "404 Not Found\r\n"),
            Self::MethodNotAllowed => write!(f, "405 Method Not Allowed\r\n"),
            Self::Conflict => write!(f, "409 Conflict\r\n"),
            Self::UnprocessableContent => write!(f, "422 Unprocessable Content\r\n"),
            Self::TooManyRequests => write!(f, "429 Too Many Requests\r\n"),

            // 5xx
            Self::InternalServerError => write!(f, "500 Internal Server Error\r\n"),
            Self::NotImplemented => write!(f, "501 Not Implemented\r\n"),
            Self::ServiceUnavailable => write!(f, "503 Service Unavailable\r\n"),
            Self::GatewayTimeout => write!(f, "504 Gateway Timeout\r\n"),
        }
    }
}
