use flate2::{Compression, write::GzEncoder};
use std::io::Write;
use tokio::io::{AsyncRead, AsyncReadExt, BufReader};
pub struct Body {
    content: Vec<u8>,
}

impl Body {
    pub fn new(content: Vec<u8>) -> Self {
        Self { content }
    }
    pub async fn parse(
        f: &mut BufReader<impl AsyncRead + Unpin>,
        length: &str,
    ) -> Result<Body, BodyError> {
        let count = length
            .parse::<usize>()
            .map_err(|_| BodyError::InvalidContentLength)?;

        let mut content = vec![0u8; count];
        let _ = f
            .read_exact(&mut content)
            .await
            .map_err(|_| BodyError::MissingData);

        if f.buffer().len() > 0 {
            return Err(BodyError::TooMushData);
        }

        Ok(Body { content })
    }


    pub fn as_bytes(&self) -> &[u8] {
        &self.content
    }

    pub fn set(&mut self, content: Vec<u8>) {
        self.content = content;
    }

    pub fn to_string_lossy(&self) -> String {
        String::from_utf8_lossy(&self.content).to_string()
    }

    pub fn len(&self) -> usize {
        self.content.len()
    }
}

#[derive(Debug, PartialEq)]
pub enum BodyError {
    InvalidContentLength,
    MissingData,
    TooMushData,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn standard_body() {
        match Body::parse(&mut BufReader::new("hello world!\n".as_bytes()), "13").await {
            Ok(b) => assert_eq!(b.to_string_lossy(), "hello world!\n"),
            Err(e) => panic!("dont pass {:?}", e),
        }
    }

    #[tokio::test]
    async fn too_mush_length_standard_body() {
        match Body::parse(&mut BufReader::new("hello world!\n".as_bytes()), "12").await {
            Ok(_) => panic!("should not pass"),
            Err(e) => assert_eq!(e, BodyError::TooMushData),
        }
    }
}
