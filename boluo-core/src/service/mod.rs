mod and_then;
mod boxed;
mod ext;
mod map_err;
mod map_request;
mod map_response;
mod map_result;
mod or_else;
mod service_fn;
mod then;

use std::future::Future;

pub use and_then::AndThen;
pub use boxed::{ArcService, BoxCloneService, BoxService};
pub use ext::ServiceExt;
pub use map_err::MapErr;
pub use map_request::MapRequest;
pub use map_response::MapResponse;
pub use map_result::MapResult;
pub use or_else::OrElse;
pub use service_fn::{service_fn, ServiceFn};
pub use then::Then;

use std::sync::Arc;

pub trait Service<Req>: Send + Sync {
    type Response;
    type Error;

    fn call(&self, req: Req) -> impl Future<Output = Result<Self::Response, Self::Error>> + Send;
}

impl<S, Req> Service<Req> for &mut S
where
    S: Service<Req> + ?Sized,
    Req: Send,
{
    type Response = S::Response;
    type Error = S::Error;

    async fn call(&self, req: Req) -> Result<Self::Response, Self::Error> {
        S::call(self, req).await
    }
}

impl<S, Req> Service<Req> for &S
where
    S: Service<Req> + ?Sized,
    Req: Send,
{
    type Response = S::Response;
    type Error = S::Error;

    async fn call(&self, req: Req) -> Result<Self::Response, Self::Error> {
        S::call(self, req).await
    }
}

impl<S, Req> Service<Req> for Box<S>
where
    S: Service<Req> + ?Sized,
    Req: Send,
{
    type Response = S::Response;
    type Error = S::Error;

    async fn call(&self, req: Req) -> Result<Self::Response, Self::Error> {
        S::call(self, req).await
    }
}

impl<S, Req> Service<Req> for Arc<S>
where
    S: Service<Req> + ?Sized,
    Req: Send,
{
    type Response = S::Response;
    type Error = S::Error;

    async fn call(&self, req: Req) -> Result<Self::Response, Self::Error> {
        S::call(self, req).await
    }
}
