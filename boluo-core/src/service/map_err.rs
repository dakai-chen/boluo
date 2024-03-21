use super::Service;

/// [`map_err`]返回的服务。
///
/// [`map_err`]: crate::service::ServiceExt::map_err
#[derive(Clone, Copy)]
pub struct MapErr<S, F> {
    service: S,
    f: F,
}

impl<S, F> MapErr<S, F> {
    /// 创建一个新的[`MapErr`]服务。
    pub fn new(service: S, f: F) -> Self {
        Self { service, f }
    }
}

impl<S, F, Req, Err> Service<Req> for MapErr<S, F>
where
    S: Service<Req>,
    F: Fn(S::Error) -> Err + Send + Sync,
    Req: Send,
{
    type Response = S::Response;
    type Error = Err;

    async fn call(&self, req: Req) -> Result<Self::Response, Self::Error> {
        match self.service.call(req).await {
            Ok(res) => Ok(res),
            Err(err) => Err((self.f)(err)),
        }
    }
}

impl<S, F> std::fmt::Debug for MapErr<S, F>
where
    S: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MapErr")
            .field("service", &self.service)
            .field("f", &std::any::type_name::<F>())
            .finish()
    }
}
