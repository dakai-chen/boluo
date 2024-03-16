use super::Service;

#[derive(Clone, Copy)]
pub struct MapResponse<S, F> {
    service: S,
    f: F,
}

impl<S, F> MapResponse<S, F> {
    pub fn new(service: S, f: F) -> Self {
        Self { service, f }
    }
}

impl<S, F, Req, Res> Service<Req> for MapResponse<S, F>
where
    S: Service<Req>,
    F: Fn(S::Response) -> Res + Send + Sync,
    Req: Send,
{
    type Response = Res;
    type Error = S::Error;

    async fn call(&self, req: Req) -> Result<Self::Response, Self::Error> {
        match self.service.call(req).await {
            Ok(res) => Ok((self.f)(res)),
            Err(err) => Err(err),
        }
    }
}

impl<S, F> std::fmt::Debug for MapResponse<S, F>
where
    S: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MapResponse")
            .field("service", &self.service)
            .field("f", &std::any::type_name::<F>())
            .finish()
    }
}
