use super::Middleware;

/// 将给定的闭包转换为[`Middleware`]。
///
/// # 例子
///
/// ```
/// use boluo_core::handler::handler_fn;
/// use boluo_core::middleware::middleware_fn;
/// use boluo_core::request::Request;
/// use boluo_core::service::{Service, ServiceExt};
///
/// fn add_extension<S>(service: S) -> impl Service<Request>
/// where
///     S: Service<Request>,
/// {
///     service.map_request(|mut req: Request| {
///         req.extensions_mut().insert("My Extension");
///         req
///     })
/// }
///
/// let service = handler_fn(|| async {});
/// let service = service.with(middleware_fn(add_extension));
/// ```
pub fn middleware_fn<F>(f: F) -> MiddlewareFn<F> {
    MiddlewareFn { f }
}

/// 将给定的闭包转换为[`Middleware`]。
#[derive(Clone, Copy)]
pub struct MiddlewareFn<F> {
    f: F,
}

impl<F, S1, S2> Middleware<S1> for MiddlewareFn<F>
where
    F: FnOnce(S1) -> S2,
{
    type Service = S2;

    fn transform(self, service: S1) -> Self::Service {
        (self.f)(service)
    }
}

impl<F> std::fmt::Debug for MiddlewareFn<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MiddlewareFn")
            .field("f", &std::any::type_name::<F>())
            .finish()
    }
}
