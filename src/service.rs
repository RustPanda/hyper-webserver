use std::{collections::BTreeMap, future::Future, pin::Pin, sync::Arc, task::{Context, Poll}};
use handlebars::Handlebars;
use hyper::{Body, Request, Response, service::Service};
use path_tree::PathTree;
use tokio::sync::{OwnedRwLockReadGuard, OwnedRwLockWriteGuard, RwLock};

use crate::endpoint::Endpoint;


pub struct RenderService {
    path_tree: Arc<RwLock<PathTree<Box<dyn Endpoint>>>>,
    handlebars: Arc<RwLock<Handlebars<'static>>>,
}

impl Service<Request<Body>> for RenderService {
    type Response = Response<Body>;
    type Error = hyper::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _: &mut Context) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let hadnlebars = self.handlebars.clone();
        let path_tree = self.path_tree.clone();
        
        Box::pin(async move {
            let path_tree: OwnedRwLockReadGuard<PathTree<Box<dyn Endpoint>>> = path_tree.read_owned().await;
            let path = &req.uri().path().to_string();
            let (endpoint, params) = path_tree.find(path).unwrap();

            let params: BTreeMap<String, String> = params.iter().fold(BTreeMap::new(), 
            |mut map, (key, val)|
            {
                map.insert(String::from(key.clone()), String::from(val.clone()));
                map
            });

            let response = endpoint.call(hadnlebars, req, params).await;
            Ok(response)
        })
    }
    
}

pub struct MakeRenderService{
    path_tree: Arc<RwLock<PathTree<Box<dyn Endpoint>>>>,
    handlebars: Arc<RwLock<Handlebars<'static>>>

}

impl MakeRenderService {
    pub fn new() -> Self {
        Self {
            path_tree: Arc::new(RwLock::new(PathTree::new())),
            handlebars: Arc::new(RwLock::new(Handlebars::new()))
        }
    }

    pub async fn router(&mut self, path: &str, endpoint: impl Endpoint) -> &mut Self {
        let path_tree = self.path_tree.clone();
        let mut path_tree:  OwnedRwLockWriteGuard<PathTree<Box<dyn Endpoint>>> = path_tree.write_owned().await;
        (*path_tree).insert(path, Box::new(endpoint));
        self
    }
}

impl< T> Service<T> for MakeRenderService {
    type Response = RenderService;
    type Error = hyper::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _: &mut Context) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _: T) -> Self::Future {
        let path_tree = self.path_tree.clone();
        let handlebars = self.handlebars.clone();
        let fut = async move {
            Ok(Self::Response {
                path_tree,
                handlebars
            })
        };
        Box::pin(fut)
    }
}