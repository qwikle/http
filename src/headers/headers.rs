use std::{
    collections::{HashMap, hash_map::Entry},
    io::{BufWriter, Write},
};
use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};

const MAX_LINE_LENGTH: usize = 8 * 1024;
const CRLF: &str = "\r\n";
pub struct Headers {
    pub headers: HashMap<String, String>,
}

impl Headers {
    pub fn new() -> Self {
        Self {
            headers: HashMap::new(),
        }
    }

    pub async fn parse(
        reader: &mut BufReader<impl AsyncRead + Unpin>,
    ) -> Result<Headers, HeadersError> {
        let mut headers = Headers {
            headers: HashMap::new(),
        };
        let mut buffer = String::new();
        loop {
            buffer.clear();
            match reader.read_line(&mut buffer).await {
                Ok(0) => return Err(HeadersError::ReadError),
                Ok(bytes_read) => {
                    if buffer == CRLF {
                        break;
                    }
                    if bytes_read == MAX_LINE_LENGTH && !buffer.ends_with(CRLF) {
                        return Err(HeadersError::LineTooLong);
                    }

                    if !buffer.ends_with(CRLF) {
                        return Err(HeadersError::MalformedEndOfLine);
                    }

                    buffer = buffer.replace(CRLF, "");
                    let parts: Vec<&str> = buffer.splitn(2, ":").collect();

                    if parts.len() != 2 {
                        return Err(HeadersError::MalformedPart);
                    }

                    let field = parts[0];
                    let value = parts[1];

                    Headers::check_field(field)?;

                    let field_normalized = field.trim().to_lowercase();
                    let value_trimmed = value.trim();

                    if let Entry::Occupied(mut entry) = headers
                        .headers
                        .entry(field_normalized.trim().to_lowercase())
                    {
                        let value_string = entry.get_mut();
                        value_string.reserve(value_trimmed.len() + 2);
                        value_string.push_str(", ");
                        value_string.push_str(value_trimmed);
                    } else {
                        headers
                            .headers
                            .insert(field_normalized, value_trimmed.to_string());
                    }
                }
                Err(_) => return Err(HeadersError::ReadError),
            }
        }
        Ok(headers)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buffer = Vec::new();

        for (key, value) in self.headers.iter() {
            let header_line = format!("{}: {}\r\n", key, value);
            buffer.extend_from_slice(header_line.as_bytes());
        }
        buffer.extend_from_slice(format!("\r\n").as_bytes());
        buffer
    }

    pub fn set(&mut self, key: &str, value: &str) -> Result<(), HeadersError> {
        Headers::check_field(key)?;
        if let Entry::Occupied(mut entry) = self.headers.entry(key.trim().to_lowercase()) {
            let value_string = entry.get_mut();
            value_string.reserve(value.len() + 2);
            value_string.push_str(", ");
            value_string.push_str(value);
            Ok(())
        } else {
            self.headers
                .insert(key.trim().to_lowercase(), value.trim().to_string());
            Ok(())
        }
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.headers.get(key.to_lowercase().as_str())
    }

    fn check_field(field: &str) -> Result<(), HeadersError> {
        if field.ends_with(" ") {
            return Err(HeadersError::MalformedFieldName);
        }

        if !field.trim().chars().all(|c| {
            c.is_ascii_alphanumeric()
                || matches!(
                    c,
                    '!' | '#'
                        | '$'
                        | '%'
                        | '&'
                        | '\''
                        | '*'
                        | '+'
                        | '-'
                        | '.'
                        | '^'
                        | '_'
                        | '`'
                        | '|'
                        | '~'
                )
        }) {
            return Err(HeadersError::MalformedFieldName);
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq)]
pub enum HeadersError {
    LineTooLong,
    MalformedEndOfLine,
    MalformedPart,
    MalformedFieldName,
    ReadError,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn valid_single_header() {
        match Headers::parse(&mut BufReader::new(
            "Host: localhost:42069\r\n\r\n".as_bytes(),
        ))
        .await
        {
            Ok(v) => {
                assert_eq!(v.headers["host"], "localhost:42069");
            }
            Err(_) => panic!("error"),
        }
    }

    #[tokio::test]
    async fn invalid_space_header() {
        match Headers::parse(&mut BufReader::new(
            "       Host : localhost:42069       \r\n\r\n".as_bytes(),
        ))
        .await
        {
            Ok(_) => panic!("should not pass"),
            Err(e) => assert_eq!(e, HeadersError::MalformedFieldName),
        }
    }

    #[tokio::test]
    async fn invalid_character_header() {
        match Headers::parse(&mut BufReader::new(
            "H©st: localhost:42069\r\n\r\n".as_bytes(),
        ))
        .await
        {
            Ok(_) => panic!("should not pass"),
            Err(e) => assert_eq!(e, HeadersError::MalformedFieldName),
        }
    }

    #[tokio::test]
    async fn valid_multiple_header() {
        match Headers::parse(&mut BufReader::new(
            "Host: localhost:42069\r\nUser-Agent: curl/8.5.0\r\n\r\n".as_bytes(),
        ))
        .await
        {
            Ok(v) => {
                assert_eq!(v.headers["host"], "localhost:42069");
                assert_eq!(v.headers["user-agent"], "curl/8.5.0")
            }
            Err(_) => panic!("error"),
        }
    }

    #[tokio::test]
    async fn valid_multiple_same_header() {
        match Headers::parse(&mut BufReader::new(
            "Host: localhost:42069\r\nHost: localhost:3333\r\n\r\n".as_bytes(),
        ))
        .await
        {
            Ok(v) => {
                assert_eq!(v.headers["host"], "localhost:42069, localhost:3333");
            }
            Err(_) => panic!("error"),
        }
    }
}
