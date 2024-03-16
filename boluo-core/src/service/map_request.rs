use super::Service;

#[derive(Clone, Copy)]
pub struct MapRequest<S, F> {
    service: S,
    f: F,
}

impl<S, F> MapRequest<S, F> {
    pub fn new(service: S, f: F) -> Self {
        Self { service, f }
    }
}

impl<S, F, R1, R2> Service<R1> for MapRequest<S, F>
where
    S: Service<R2>,
    F: Fn(R1) -> R2 + Send + Sync,
    R1: Send,
    R2: Send,
{
    type Response = S::Response;
    type Error = S::Error;

    async fn call(&self, req: R1) -> Result<Self::Response, Self::Error> {
        self.service.call((self.f)(req)).await
    }
}

impl<S, F> std::fmt::Debug for MapRequest<S, F>
where
    S: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MapRequest")
            .field("service", &self.service)
            .field("f", &std::any::type_name::<F>())
            .finish()
    }
}
