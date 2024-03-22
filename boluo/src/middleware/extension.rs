use std::future::Future;

use boluo_core::middleware::Middleware;
use boluo_core::request::Request;
use boluo_core::service::Service;

pub use crate::extract::Extension;

impl<S, T> Middleware<S> for Extension<T> {
    type Service = ExtensionService<S, T>;

    fn transform(self, service: S) -> Self::Service {
        ExtensionService {
            service,
            value: Extension::into_inner(self),
        }
    }
}

/// 中间件[`Extension`]返回的服务。
#[derive(Debug, Clone, Copy)]
pub struct ExtensionService<S, T> {
    service: S,
    value: T,
}

impl<B, S, T> Service<Request<B>> for ExtensionService<S, T>
where
    S: Service<Request<B>>,
    T: Clone + Send + Sync + 'static,
{
    type Response = S::Response;
    type Error = S::Error;

    fn call(
        &self,
        mut req: Request<B>,
    ) -> impl Future<Output = Result<Self::Response, Self::Error>> + Send {
        req.extensions_mut().insert(self.value.clone());
        self.service.call(req)
    }
}
