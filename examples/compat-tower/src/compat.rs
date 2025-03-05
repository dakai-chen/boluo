use std::future::poll_fn;
use std::pin::Pin;
use std::task::{Context, Poll};

use boluo::middleware::Middleware;
use boluo::request::Request;
use boluo::response::Response;
use boluo::service::Service;
use tower::Layer;

#[derive(Debug, Clone, Copy)]
pub struct CompatTower<S>(pub S);

impl<R, S> Service<R> for CompatTower<S>
where
    S: tower::Service<R> + Clone + Send + Sync,
    S::Future: Send,
    R: Send,
{
    type Response = S::Response;
    type Error = S::Error;

    async fn call(&self, req: R) -> Result<Self::Response, Self::Error> {
        let mut service = self.0.clone();
        poll_fn(|cx| service.poll_ready(cx)).await?;
        service.call(req).await
    }
}

impl<S, L> Middleware<S> for CompatTower<L>
where
    L: Layer<TowerService<S>>,
{
    type Service = CompatTower<L::Service>;

    fn transform(self, service: S) -> Self::Service {
        CompatTower(self.0.layer(TowerService(service)))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CompatTowerHttp<S>(pub S);

impl<ReqB, ResB, S> Service<Request<ReqB>> for CompatTowerHttp<S>
where
    S: tower::Service<http::Request<ReqB>, Response = http::Response<ResB>> + Clone + Send + Sync,
    S::Future: Send,
    ReqB: Send,
{
    type Response = Response<ResB>;
    type Error = S::Error;

    async fn call(&self, req: Request<ReqB>) -> Result<Self::Response, Self::Error> {
        let req = into_http_request(req);
        let mut service = self.0.clone();
        poll_fn(|cx| service.poll_ready(cx)).await?;
        service.call(req).await.map(into_boluo_response)
    }
}

impl<S, L> Middleware<S> for CompatTowerHttp<L>
where
    L: Layer<TowerHttpService<S>>,
{
    type Service = CompatTowerHttp<L::Service>;

    fn transform(self, service: S) -> Self::Service {
        CompatTowerHttp(self.0.layer(TowerHttpService(service)))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TowerService<S>(pub S);

impl<R, S> tower::Service<R> for TowerService<S>
where
    S: Service<R> + Clone + 'static,
    R: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: R) -> Self::Future {
        let service = self.0.clone();
        Box::pin(async move { service.call(req).await })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TowerHttpService<S>(pub S);

impl<ReqB, ResB, S> tower::Service<http::Request<ReqB>> for TowerHttpService<S>
where
    S: Service<Request<ReqB>, Response = Response<ResB>> + Clone + 'static,
    ReqB: Send + 'static,
{
    type Response = http::Response<ResB>;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: http::Request<ReqB>) -> Self::Future {
        let req = into_boluo_request(req);
        let service = self.0.clone();
        Box::pin(async move { service.call(req).await.map(into_http_response) })
    }
}

fn into_http_request<B>(req: Request<B>) -> http::Request<B> {
    let (parts, body) = req.into_inner();
    let mut req = http::Request::new(body);
    *req.method_mut() = parts.method;
    *req.uri_mut() = parts.uri;
    *req.version_mut() = parts.version;
    *req.headers_mut() = parts.headers;
    *req.extensions_mut() = parts.extensions;
    req
}

fn into_http_response<B>(res: Response<B>) -> http::Response<B> {
    let (parts, body) = res.into_inner();
    let mut res = http::Response::new(body);
    *res.status_mut() = parts.status;
    *res.version_mut() = parts.version;
    *res.headers_mut() = parts.headers;
    *res.extensions_mut() = parts.extensions;
    res
}

fn into_boluo_request<B>(req: http::Request<B>) -> Request<B> {
    let (parts, body) = req.into_parts();
    let mut req = Request::new(body);
    *req.method_mut() = parts.method;
    *req.uri_mut() = parts.uri;
    *req.version_mut() = parts.version;
    *req.headers_mut() = parts.headers;
    *req.extensions_mut() = parts.extensions;
    req
}

fn into_boluo_response<B>(res: http::Response<B>) -> Response<B> {
    let (parts, body) = res.into_parts();
    let mut res = Response::new(body);
    *res.status_mut() = parts.status;
    *res.version_mut() = parts.version;
    *res.headers_mut() = parts.headers;
    *res.extensions_mut() = parts.extensions;
    res
}
