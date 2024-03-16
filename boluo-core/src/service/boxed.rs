use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use super::Service;

type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub struct BoxService<Req, Res, Err> {
    service: Box<dyn AnyService<Req, Response = Res, Error = Err>>,
}

impl<Req, Res, Err> BoxService<Req, Res, Err> {
    pub fn new<S>(service: S) -> Self
    where
        S: Service<Req, Response = Res, Error = Err> + 'static,
    {
        Self {
            service: Box::new(service),
        }
    }
}

impl<Req, Res, Err> Service<Req> for BoxService<Req, Res, Err>
where
    Req: Send,
{
    type Response = Res;
    type Error = Err;

    async fn call(&self, req: Req) -> Result<Self::Response, Self::Error> {
        self.service.call(req).await
    }
}

impl<Req, Res, Err> std::fmt::Debug for BoxService<Req, Res, Err> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BoxService").finish()
    }
}

pub struct ArcService<Req, Res, Err> {
    service: Arc<dyn AnyService<Req, Response = Res, Error = Err>>,
}

impl<Req, Res, Err> ArcService<Req, Res, Err> {
    pub fn new<S>(service: S) -> Self
    where
        S: Service<Req, Response = Res, Error = Err> + 'static,
    {
        Self {
            service: Arc::new(service),
        }
    }
}

impl<Req, Res, Err> Service<Req> for ArcService<Req, Res, Err>
where
    Req: Send,
{
    type Response = Res;
    type Error = Err;

    async fn call(&self, req: Req) -> Result<Self::Response, Self::Error> {
        self.service.call(req).await
    }
}

impl<Req, Res, Err> Clone for ArcService<Req, Res, Err> {
    fn clone(&self) -> Self {
        Self {
            service: Arc::clone(&self.service),
        }
    }
}

impl<Req, Res, Err> std::fmt::Debug for ArcService<Req, Res, Err> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ArcService").finish()
    }
}

pub struct BoxCloneService<Req, Res, Err> {
    service: Box<dyn CloneService<Req, Response = Res, Error = Err>>,
}

impl<Req, Res, Err> BoxCloneService<Req, Res, Err> {
    pub fn new<S>(service: S) -> Self
    where
        S: Service<Req, Response = Res, Error = Err> + Clone + 'static,
    {
        Self {
            service: Box::new(service),
        }
    }
}

impl<Req, Res, Err> Service<Req> for BoxCloneService<Req, Res, Err>
where
    Req: Send,
{
    type Response = Res;
    type Error = Err;

    async fn call(&self, req: Req) -> Result<Self::Response, Self::Error> {
        self.service.call(req).await
    }
}

impl<Req, Res, Err> Clone for BoxCloneService<Req, Res, Err> {
    fn clone(&self) -> Self {
        Self {
            service: self.service.clone_box(),
        }
    }
}

impl<Req, Res, Err> std::fmt::Debug for BoxCloneService<Req, Res, Err> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BoxCloneService").finish()
    }
}

trait CloneService<Req>: AnyService<Req> {
    fn clone_box(
        &self,
    ) -> Box<dyn CloneService<Req, Response = Self::Response, Error = Self::Error>>;
}

impl<S, Req> CloneService<Req> for S
where
    S: Service<Req> + ?Sized + Clone + 'static,
{
    fn clone_box(
        &self,
    ) -> Box<dyn CloneService<Req, Response = Self::Response, Error = Self::Error>> {
        Box::new(self.clone())
    }
}

trait AnyService<Req>: Send + Sync {
    type Response;
    type Error;

    fn call<'a>(&'a self, req: Req) -> BoxFuture<'a, Result<Self::Response, Self::Error>>
    where
        Req: 'a;
}

impl<S, Req> AnyService<Req> for S
where
    S: Service<Req> + ?Sized,
{
    type Response = S::Response;
    type Error = S::Error;

    fn call<'a>(&'a self, req: Req) -> BoxFuture<'a, Result<Self::Response, Self::Error>>
    where
        Req: 'a,
    {
        Box::pin(Service::call(self, req))
    }
}
