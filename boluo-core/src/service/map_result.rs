use super::Service;

#[derive(Clone, Copy)]
pub struct MapResult<S, F> {
    service: S,
    f: F,
}

impl<S, F> MapResult<S, F> {
    pub fn new(service: S, f: F) -> Self {
        Self { service, f }
    }
}

impl<S, F, Req, Res, Err> Service<Req> for MapResult<S, F>
where
    S: Service<Req>,
    F: Fn(Result<S::Response, S::Error>) -> Result<Res, Err> + Send + Sync,
    Req: Send,
{
    type Response = Res;
    type Error = Err;

    async fn call(&self, req: Req) -> Result<Self::Response, Self::Error> {
        (self.f)(self.service.call(req).await)
    }
}

impl<S, F> std::fmt::Debug for MapResult<S, F>
where
    S: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MapResult")
            .field("service", &self.service)
            .field("f", &std::any::type_name::<F>())
            .finish()
    }
}
