use std::{fmt, future::Future};

use super::Service;

/// 将给定的异步函数转换为[`Service`]。
///
/// # 例子
///
/// ```
/// use std::convert::Infallible;
///
/// use boluo_core::service::service_fn;
///
/// async fn hello(_: ()) -> Result<&'static str, Infallible> {
///     Ok("Hello, World!")
/// }
///
/// let service = service_fn(hello);
/// ```
pub fn service_fn<F>(f: F) -> ServiceFn<F> {
    ServiceFn { f }
}

/// 将给定的异步函数转换为[`Service`]。
#[derive(Clone, Copy)]

pub struct ServiceFn<F> {
    f: F,
}

impl<F, Fut, Req, Res, Err> Service<Req> for ServiceFn<F>
where
    F: Fn(Req) -> Fut + Send + Sync,
    Fut: Future<Output = Result<Res, Err>> + Send,
    Req: Send,
{
    type Response = Res;
    type Error = Err;

    async fn call(&self, req: Req) -> Result<Self::Response, Self::Error> {
        (self.f)(req).await
    }
}

impl<F> fmt::Debug for ServiceFn<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ServiceFn")
            .field("f", &std::any::type_name::<F>())
            .finish()
    }
}
