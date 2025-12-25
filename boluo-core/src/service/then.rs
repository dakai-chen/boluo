use super::Service;

/// [`then`] 返回的服务。
///
/// [`then`]: crate::service::ServiceExt::then
#[derive(Clone, Copy)]
pub struct Then<S, F> {
    service: S,
    f: F,
}

impl<S, F> Then<S, F> {
    /// 创建一个新的 [`Then`] 服务。
    pub fn new(service: S, f: F) -> Self {
        Self { service, f }
    }
}

impl<S, F, Fut, Req, Res, Err> Service<Req> for Then<S, F>
where
    S: Service<Req>,
    F: Fn(Result<S::Response, S::Error>) -> Fut + Send + Sync,
    Fut: Future<Output = Result<Res, Err>> + Send,
{
    type Response = Res;
    type Error = Err;

    fn call(
        &self,
        request: Req,
    ) -> impl Future<Output = Result<Self::Response, Self::Error>> + Send {
        let fut = self.service.call(request);
        async move { (self.f)(fut.await).await }
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
