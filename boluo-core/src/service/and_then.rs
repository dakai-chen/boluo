use std::future::Future;

use super::Service;

#[derive(Clone, Copy)]
pub struct AndThen<S, F> {
    service: S,
    f: F,
}

impl<S, F> AndThen<S, F> {
    pub fn new(service: S, f: F) -> Self {
        Self { service, f }
    }
}

impl<S, F, Fut, Req, Res> Service<Req> for AndThen<S, F>
where
    S: Service<Req>,
    S::Response: Send,
    F: Fn(S::Response) -> Fut + Send + Sync,
    Fut: Future<Output = Result<Res, S::Error>> + Send,
    Req: Send,
{
    type Response = Res;
    type Error = S::Error;

    async fn call(&self, req: Req) -> Result<Self::Response, Self::Error> {
        let response = self.service.call(req).await?;
        (self.f)(response).await
    }
}

impl<S, F> std::fmt::Debug for AndThen<S, F>
where
    S: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AndThen")
            .field("service", &self.service)
            .field("f", &std::any::type_name::<F>())
            .finish()
    }
}
