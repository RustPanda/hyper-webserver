use async_trait::async_trait;
use handlebars::Handlebars;
use hyper::{Body, Request, Response};
use tokio::sync::RwLock;
use std::{collections::BTreeMap, future::Future, sync::Arc};

///Позваляет передовать и сохранять async Fn() -> String
#[async_trait]
pub trait Endpoint: Sync + Send + 'static {
   async fn call(&self,  hb: Arc<RwLock<Handlebars<'static>>>, req: Request<Body>, params: BTreeMap<String, String>) -> Response<Body>;
}

#[async_trait]
impl <F, Fut> Endpoint for F where
    F: Copy + Send + Sync + 'static + FnOnce(Arc<RwLock<Handlebars<'static>>>, Request<Body>, BTreeMap<String, String>) -> Fut,
    Fut: Future<Output = Response<Body>> + Send + 'static {
        async fn call(&self, handlebars: Arc<RwLock<Handlebars<'static>>>, req: Request<Body>, params: BTreeMap<String, String>) -> Response<Body> {
            let fut = self(handlebars, req, params);
            fut.await   
    }
}
