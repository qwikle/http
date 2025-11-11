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
async fn main() -> std::io::Result<()> {
    let mut router = Router::new();
    router.get("/", hello_handler);
    router.get("/hello", by_handler);
    let server = Server::serve("3333", router).await.unwrap();

    server.listen().await;
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
