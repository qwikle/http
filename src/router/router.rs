use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use tokio::io::{AsyncWriteExt, WriteHalf};
use tokio::net::TcpStream;
use tokio::sync::{Mutex, oneshot};

use crate::request::request::Request;
use crate::response::Response;
use crate::response::response::Status;

pub type HandlerResult = Result<Response, HandlerError>;
pub type AsyncHandler =
    Box<dyn Fn(Context) -> Pin<Box<dyn Future<Output = HandlerResult> + Send>> + Send + Sync>;

pub struct Router {
    routes: HashMap<String, AsyncHandler>,
}

pub struct Context {
    pub request: Request,
    pub response: Response,
}

#[derive(Debug)]
pub enum HandlerError {
    NotFound,
    InternalError,
}

impl Router {
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
        }
    }

    pub fn add_route<F, Fut>(&mut self, path: &str, handler: F)
    where
        F: Fn(Context) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HandlerResult> + Send + 'static,
    {
        let boxed_handler = Box::new(move |ctx| {
            Box::pin(handler(ctx)) as Pin<Box<dyn Future<Output = HandlerResult> + Send>>
        });

        self.routes.insert(path.to_string(), boxed_handler);
    }

    pub fn get<F, Fut>(&mut self, path: &str, handler: F)
    where
        F: Fn(Context) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HandlerResult> + Send + 'static,
    {
        self.add_route(path, handler);
    }

    pub fn post<F, Fut>(&mut self, path: &str, handler: F)
    where
        F: Fn(Context) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HandlerResult> + Send + 'static,
    {
        self.add_route(path, handler);
    }

    pub async fn handle_request(
        &self,
        request: Request,
        mut response: Response,
        mut wr: WriteHalf<TcpStream>,
    ) {
        let path = &request.request_line.request_target;

        if let Some(handler) = self.routes.get(path) {
            let context = Context { request, response };

            let rc = &context.request.clone();
            let mut res = handler(context).await.unwrap();
            let encoding = rc.headers.get("Accept-Encoding");
            wr.write_all(&res.send(encoding)).await.unwrap();
        } else {
            response.status(Status::NotFound);
        }
    }
}
