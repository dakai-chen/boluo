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
