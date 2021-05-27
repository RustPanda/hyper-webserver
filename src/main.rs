use handlebars::Handlebars;
use hyper::{service::Service, Body, Request, Response, Server};
use path_tree::PathTree;
use rhai::{Dynamic, Engine, Scope};
use std::{
    future::Future,
    net::SocketAddr,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};
use tokio::sync::{OwnedRwLockReadGuard, OwnedRwLockWriteGuard, RwLock};

#[tokio::main]
async fn main() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));

    let mut router = PathTree::new();
    router.insert("/", "index".into());


    let mut template = Handlebars::new();
    template.register_template_file("index", "./templates/index.html").unwrap();
    let _= template.register_template_file("404", "./templates/404.html");
    let engine = Engine::new();

    let server = Server::bind(&addr).serve(MakeRender::new(router, template, engine));
    let graceful = server.with_graceful_shutdown(async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install CTRL+C signal handler")
    });

    if let Err(e) = graceful.await {
        eprintln!("server error: {}", e);
    }
}

struct Render {
    router: PathTree<String>,
    template: Arc<RwLock<Handlebars<'static>>>,
    engine: Arc<RwLock<Engine>>,
}

impl Service<Request<Body>> for Render {
    type Response = Response<Body>;
    type Error = hyper::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _: &mut Context) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let template = self.template.clone();
        let router = self.router.clone();
        let engine = self.engine.clone();

        Box::pin(async move {

            let path = req.uri().path();
            let guars_tmp: OwnedRwLockReadGuard<Handlebars> = template.read_owned().await;

            if let Some((template, params)) = router.find(path) {
                if let Ok(body) = (*guars_tmp).render(template, &params) {
                    return Ok(Response::builder().status(200).body(Body::from(body)).unwrap())
                }
            } else if let Ok(body) = (*guars_tmp).render("404", &Dynamic::ZERO) {
                return Ok(Response::builder().status(200).body(Body::from(body)).unwrap())
            }

            Ok(Response::builder().status(404).body(Body::from("Not Found")).unwrap())
        })
    }
}

struct MakeRender{
    router: PathTree<String>,
    template: Arc<RwLock<Handlebars<'static>>>,
    engine: Arc<RwLock<Engine>>,

}

impl MakeRender {
    fn new(router: PathTree<String>, template: Handlebars<'static>, rhai: Engine) -> Self {
        Self {
            router,
            template: Arc::new(RwLock::new(template)),
            engine: Arc::new(RwLock::new(rhai)),
        }
    }
}

impl< T> Service<T> for MakeRender {
    type Response = Render;
    type Error = hyper::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _: &mut Context) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _: T) -> Self::Future {
        let router = self.router.clone();
        let template = self.template.clone();
        let engine = self.engine.clone();
        let fut = async move {
            Ok(Render {
                router,
                template,
                engine,
            })
        };
        Box::pin(fut)
    }
}
