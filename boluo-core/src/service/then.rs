use std::future::Future;

use super::Service;

#[derive(Clone, Copy)]
pub struct Then<S, F> {
    service: S,
    f: F,
}

impl<S, F> Then<S, F> {
    pub fn new(service: S, f: F) -> Self {
        Self { service, f }
    }
}

impl<S, F, Fut, Req, Res, Err> Service<Req> for Then<S, F>
where
    S: Service<Req>,
    S::Response: Send,
    S::Error: Send,
    F: Fn(Result<S::Response, S::Error>) -> Fut + Send + Sync,
    Fut: Future<Output = Result<Res, Err>> + Send,
    Req: Send,
{
    type Response = Res;
    type Error = Err;

    async fn call(&self, req: Req) -> Result<Self::Response, Self::Error> {
        (self.f)(self.service.call(req).await).await
    }
}

impl<S, F> std::fmt::Debug for Then<S, F>
where
    S: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Then")
            .field("service", &self.service)
            .field("f", &std::any::type_name::<F>())
            .finish()
    }
}
