use std::{net::SocketAddr, sync::{Arc}};
pub use crate::server::ServerState;
use tokio::{
    io, net::{TcpListener, TcpStream},
};
use tracing::{error, info, warn};

use crate::{
    request::request_from_reader,
    response::{Response},
    router::router::Router, server::lifecycle::LifecycleManager,
};

pub struct Server {
    listener: Option<TcpListener>,
    lifecycle: LifecycleManager,
    router: Arc<Router>
}
    
impl Server {
    
    pub async fn new(router: Router) -> Result<Self, Box<dyn std::error::Error>> {
        let server = Self {
          lifecycle: LifecycleManager::new(),
          router: Arc::new(router),
        listener: None  
        };
        
        server.boot().await?;
        Ok(server)
    }
    
    async fn boot(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Booting server...");
        
        
        self.lifecycle.transition_to(ServerState::Ready)?;
        info!("Server booted..");
        Ok(())
    }
    
    pub async fn start(&mut self, addr: &str) -> Result<(), Box<dyn std::error::Error>> {
        if self.lifecycle.curent_state() != ServerState::Ready {
            return Err("Server must be in Ready state.".into());
        }
        self.listener =  Some(TcpListener::bind(addr).await?);
        self.lifecycle.transition_to(ServerState::Run)?;
        info!("Server is running on {addr}");
        self.listen().await
    }

     async fn listen(&self)-> Result<(), Box<dyn std::error::Error>> {
        let listener = self.listener.as_ref().unwrap();
        
        while self.lifecycle.curent_state() == ServerState::Run {
            tokio::select! {
                accept_result = listener.accept()=> {
                    match accept_result {
                        Ok((socket, addr)) => {
                            self.handle_connection(socket, addr).await;
                        },
                        Err(e) => error!("Accept Error: {e}")
                    }
                }
                _ = self.wait_for_shutdown() => {
                    break;
                }
            }
        }
        self.shutdown().await
    }
    
    async fn handle_connection(&self, socket: TcpStream, addr: SocketAddr) {
        if self.lifecycle.curent_state() != ServerState::Run {
            error!("Connexion rejected - server is shutting down");
            return;
        }
        static CONNECTION_SEMAPHORE: tokio::sync::Semaphore = tokio::sync::Semaphore::const_new(10000);
        
        let permit = match CONNECTION_SEMAPHORE.acquire().await {
            Ok(p) =>p,
            Err(_) => {
                error!("too many simultaneous connections, {} rejected", addr);
                return;
            }  
        };
        
        let router = Arc::clone(&self.router);
        tokio::spawn(async move{
            let _permit = permit;
            
            match Self::process_connection(socket, router).await {
                Ok(_) => {
                info!("Connection ended with success: {}",addr);
                },
                Err(e) => {
                warn!("Erreur on the connection {}: {}",addr, e);  
                }
            }
        });
    }
    
    async fn process_connection( socket: TcpStream, router: Arc<Router>) -> Result<(),Box<dyn std::error::Error>> {
        let (rd, rw) = io::split(socket);
        
        let request = request_from_reader(rd).await.unwrap();
        let router_guard = router;
        router_guard.handle_request(request, Response::new(), rw).await;
        Ok(())
    }
    
    async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.lifecycle.transition_to(ServerState::Closing)?;
        self.close_connections().await?;
        self.cleanup().await?;
        self.lifecycle.transition_to(ServerState::Shutdown)?;
        info!("Shutting down completes");
        Ok(())
    }
    
    async fn wait_for_shutdown(&self) {
        tokio::signal::ctrl_c().await.expect("Cannot read signal.")
    }
    
    
    async fn close_connections(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
    
    async fn cleanup(&self)-> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}



#[derive(Debug, PartialEq)]
pub enum ServerError {
    PortAlReadyUsed,
}
