use super::Service;

/// [`map_result`] 返回的服务。
///
/// [`map_result`]: crate::service::ServiceExt::map_result
#[derive(Clone, Copy)]
pub struct MapResult<S, F> {
    service: S,
    f: F,
}

impl<S, F> MapResult<S, F> {
    /// 创建一个新的 [`MapResult`] 服务。
    pub fn new(service: S, f: F) -> Self {
        Self { service, f }
    }
}

impl<S, F, Req, Res, Err> Service<Req> for MapResult<S, F>
where
    S: Service<Req>,
    F: Fn(Result<S::Response, S::Error>) -> Result<Res, Err> + Send + Sync,
{
    type Response = Res;
    type Error = Err;

    fn call(&self, req: Req) -> impl Future<Output = Result<Self::Response, Self::Error>> + Send {
        let fut = self.service.call(req);
        async move { (self.f)(fut.await) }
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
