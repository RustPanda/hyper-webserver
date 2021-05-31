#![feature(async_closure)]

use std::{collections::BTreeMap, net::SocketAddr, sync::Arc};
use handlebars::Handlebars;
use hyper::{Body, Request, Response, Server};
use tokio::sync::RwLock;

mod service;
mod endpoint;

#[tokio::main]
async fn main() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));

    let mut render_service = service::MakeRenderService::new();

    &mut render_service.router("/", index).await;

    let server = Server::bind(&addr).serve(render_service);
    let graceful = server.with_graceful_shutdown(async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install CTRL+C signal handler")
    });

    if let Err(e) = graceful.await {
        eprintln!("server error: {}", e);
    }
}

async fn index( _hb: Arc<RwLock<Handlebars<'static>>>, req: Request<Body>, _params: BTreeMap<String, String>) -> Response<Body> {
    let text = req.uri().path().to_string();
    Response::builder().body(Body::from(text)).unwrap()
}