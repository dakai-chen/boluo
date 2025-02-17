use super::Service;

/// [`map_response`]返回的服务。
///
/// [`map_response`]: crate::service::ServiceExt::map_response
#[derive(Clone, Copy)]
pub struct MapResponse<S, F> {
    service: S,
    f: F,
}

impl<S, F> MapResponse<S, F> {
    /// 创建一个新的[`MapResponse`]服务。
    pub fn new(service: S, f: F) -> Self {
        Self { service, f }
    }
}

impl<S, F, Req, Res> Service<Req> for MapResponse<S, F>
where
    S: Service<Req>,
    F: Fn(S::Response) -> Res + Send + Sync,
{
    type Response = Res;
    type Error = S::Error;

    fn call(&self, req: Req) -> impl Future<Output = Result<Self::Response, Self::Error>> + Send {
        let fut = self.service.call(req);
        async move { fut.await.map(|res| (self.f)(res)) }
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
