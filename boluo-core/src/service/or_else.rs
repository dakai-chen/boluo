use std::future::Future;

use super::Service;

#[derive(Clone, Copy)]
pub struct OrElse<S, F> {
    service: S,
    f: F,
}

impl<S, F> OrElse<S, F> {
    pub fn new(service: S, f: F) -> Self {
        Self { service, f }
    }
}

impl<S, F, Fut, Req, Err> Service<Req> for OrElse<S, F>
where
    S: Service<Req>,
    S::Error: Send,
    F: Fn(S::Error) -> Fut + Send + Sync,
    Fut: Future<Output = Result<S::Response, Err>> + Send,
    Req: Send,
{
    type Response = S::Response;
    type Error = Err;

    async fn call(&self, req: Req) -> Result<Self::Response, Self::Error> {
        let err = match self.service.call(req).await {
            Ok(res) => return Ok(res),
            Err(err) => err,
        };
        (self.f)(err).await
    }
}

impl<S, F> std::fmt::Debug for OrElse<S, F>
where
    S: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OrElse")
            .field("service", &self.service)
            .field("f", &std::any::type_name::<F>())
            .finish()
    }
}
