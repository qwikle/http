use crate::server::Server;
mod headers;
mod request;
mod response;
mod server;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let server = Server::serve("3333").await.unwrap();
    server.listen().await;
    Ok(())
}
