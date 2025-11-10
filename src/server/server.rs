use tokio::{
    io::{self, AsyncWriteExt},
    net::TcpListener,
};
use tracing::info;

use crate::{request::request_from_reader, response::Response};

pub struct Server {
    listener: TcpListener,
}

impl Server {
    pub async fn serve(port: &str) -> Result<Self, ServerError> {
        tracing_subscriber::fmt().init();
        let mut addr = String::from("127.0.0.1:").to_owned();
        addr.push_str(port);
        let listener = TcpListener::bind(&addr)
            .await
            .map_err(|_| ServerError::PortAlReadyUsed)?;
        info!("Server running and listening at : {addr}");
        Ok(Self { listener })
    }

    pub async fn listen(&self) {
        loop {
            let (socket, _) = self.listener.accept().await.unwrap();
            let (rd, mut wr) = io::split(socket);
            tokio::spawn(async move {
                let r = request_from_reader(rd).await.unwrap();
                let mut response = Response::new();
                let html_content = r#"
                <!DOCTYPE html>
                <html>
                <head>
                    <title>Test Page</title>
                    <style>body { font-family: Arial; margin: 20px; }</style>
                </head>
                <body>
                    <h1>Hello World!</h1>
                    <p>This is a test page with enough content to make compression worthwhile.</p>
                    <!-- Répéter ce contenu plusieurs fois pour grossir le fichier -->
                </body>
                </html>
                "#.repeat(10);
                response.body(html_content.into());
                wr.write_all(&response.send(r.headers.headers.get("accept-encoding")))
                    .await
                    .unwrap();
            });
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ServerError {
    PortAlReadyUsed,
}
