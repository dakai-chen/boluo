use std::future::Future;

use super::Service;

/// [`and_then`]返回的服务。
///
/// [`and_then`]: crate::service::ServiceExt::and_then
#[derive(Clone, Copy)]
pub struct AndThen<S, F> {
    service: S,
    f: F,
}

impl<S, F> AndThen<S, F> {
    /// 创建一个新的[`AndThen`]服务。
    pub fn new(service: S, f: F) -> Self {
        Self { service, f }
    }
}

impl<S, F, Fut, Req, Res> Service<Req> for AndThen<S, F>
where
    S: Service<Req>,
    F: Fn(S::Response) -> Fut + Send + Sync,
    Fut: Future<Output = Result<Res, S::Error>> + Send,
{
    type Response = Res;
    type Error = S::Error;

    fn call(&self, req: Req) -> impl Future<Output = Result<Self::Response, Self::Error>> + Send {
        let fut = self.service.call(req);
        async move {
            let response = fut.await?;
            (self.f)(response).await
        }
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
