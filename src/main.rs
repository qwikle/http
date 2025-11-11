use crate::{
    router::router::{Context, HandlerResult, Router},
    server::Server,
};
mod headers;
mod request;
mod response;
mod router;
mod server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let mut router = Router::new();
    router.get("/", hello_handler);
    router.get("/hello", by_handler);
    let mut server = match Server::new(router).await {
        Ok(server) =>server,
        Err(e) => {
            println!("Cannot run server {e}");
            return Err(e);
        }
    };

    match server.start("127.0.0.1:3333").await {
        Ok(_) => {
            println!("✅ Serveur arrêté proprement");
        }
        Err(e) => {
            eprintln!("❌ Erreur du serveur: {}", e);
            std::process::exit(1);
        }
    }
   Ok(())
}

async fn hello_handler(ctx: Context) -> HandlerResult {
    let mut response = ctx.response;
    response.body("Hello World".into());
    Ok(response)
}

async fn by_handler(ctx: Context) -> HandlerResult {
    let mut response = ctx.response;
    response.body("{\"Salut\": \"coupain\"}".into());
    Ok(response)
}
