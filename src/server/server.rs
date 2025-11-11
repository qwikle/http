use std::sync::Arc;

use tokio::{
    io::{self},
    net::TcpListener,
};
use tracing::info;

use crate::{
    request::request_from_reader,
    response::{Response},
    router::router::Router,
};

pub struct Server {
    listener: TcpListener,
    router: Arc<Router>,
}

impl Server {
    pub async fn serve(port: &str, router: Router) -> Result<Self, ServerError> {
        tracing_subscriber::fmt().init();
        let mut addr = String::from("127.0.0.1:").to_owned();
        addr.push_str(port);
        let listener = TcpListener::bind(&addr)
            .await
            .map_err(|_| ServerError::PortAlReadyUsed)?;
        info!("Server running and listening at : {addr}");
        Ok(Self {
            listener,
            router: Arc::new(router),
        })
    }

    pub async fn listen(&self) {
        loop {
            let (socket, _) = self.listener.accept().await.unwrap();
            let (rd, wr) = io::split(socket);
            let router = Arc::clone(&self.router);
            tokio::spawn(async move {
                if let Ok(r) = request_from_reader(rd).await {
                let response = Response::new();
                router.handle_request(r, response, wr).await;
                    
                }
            });
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ServerError {
    PortAlReadyUsed,
}
