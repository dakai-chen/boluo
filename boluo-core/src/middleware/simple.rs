use std::future::Future;

use crate::middleware::Middleware;
use crate::service::Service;

/// 将给定的异步函数转换为[`Middleware`]，并可以携带状态。
///
/// # 例子
///
/// ```
/// use boluo_core::handler::handler_fn;
/// use boluo_core::middleware::simple_middleware_fn_with_state;
/// use boluo_core::request::Request;
/// use boluo_core::service::{Service, ServiceExt};
///
/// fn assert_service<S, R>(service: S) -> S
/// where
///     S: Service<R>,
/// {
///     service
/// }
///
/// // 日志中间件
/// async fn log<S>(prefix: &&str, req: Request, service: &S) -> Result<S::Response, S::Error>
/// where
///     S: Service<Request>,
/// {
///     println!("{prefix}: {} {}", req.method(), req.uri().path());
///     service.call(req).await
/// }
///
/// let service = handler_fn(|| async {});
/// let service = service.with(simple_middleware_fn_with_state("HTTP", log));
///
/// assert_service(service);
/// ```
pub fn simple_middleware_fn_with_state<T, F>(state: T, f: F) -> SimpleMiddlewareFnWithState<T, F> {
    SimpleMiddlewareFnWithState { state, f }
}

/// 详情查看[`simple_middleware_fn_with_state`]。
#[derive(Clone, Copy)]
pub struct SimpleMiddlewareFnWithState<T, F> {
    state: T,
    f: F,
}

impl<T, F, S> Middleware<S> for SimpleMiddlewareFnWithState<T, F> {
    type Service = SimpleMiddlewareFnWithStateService<T, F, S>;

    fn transform(self, service: S) -> Self::Service {
        SimpleMiddlewareFnWithStateService {
            state: self.state,
            f: self.f,
            service,
        }
    }
}

impl<T, F> std::fmt::Debug for SimpleMiddlewareFnWithState<T, F>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimpleMiddlewareFnWithState")
            .field("state", &self.state)
            .field("f", &std::any::type_name::<F>())
            .finish()
    }
}

/// 中间件[`SimpleMiddlewareFnWithState`]返回的服务。
#[derive(Clone, Copy)]
pub struct SimpleMiddlewareFnWithStateService<T, F, S> {
    state: T,
    f: F,
    service: S,
}

impl<T, F, S, Req, Res, Err> Service<Req> for SimpleMiddlewareFnWithStateService<T, F, S>
where
    for<'a> F: SimpleMiddlewareWithState<'a, T, S, Req, Res = Res, Err = Err>,
    Self: Send + Sync,
{
    type Response = Res;
    type Error = Err;

    fn call(&self, req: Req) -> impl Future<Output = Result<Self::Response, Self::Error>> + Send {
        self.f.call(&self.state, req, &self.service)
    }
}

impl<T, F, S> std::fmt::Debug for SimpleMiddlewareFnWithStateService<T, F, S>
where
    T: std::fmt::Debug,
    S: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimpleMiddlewareFnWithStateService")
            .field("state", &self.state)
            .field("f", &std::any::type_name::<F>())
            .field("service", &self.service)
            .finish()
    }
}

trait SimpleMiddlewareWithState<'a, T, S, R>
where
    T: ?Sized,
    S: ?Sized,
{
    type Res;
    type Err;

    fn call(
        &self,
        state: &'a T,
        req: R,
        service: &'a S,
    ) -> impl Future<Output = Result<Self::Res, Self::Err>> + Send;
}

impl<'a, T, S, F, Fut, Req, Res, Err> SimpleMiddlewareWithState<'a, T, S, Req> for F
where
    T: ?Sized + 'a,
    S: ?Sized + 'a,
    F: Fn(&'a T, Req, &'a S) -> Fut,
    Fut: Future<Output = Result<Res, Err>> + Send + 'a,
{
    type Res = Res;
    type Err = Err;

    fn call(
        &self,
        state: &'a T,
        req: Req,
        service: &'a S,
    ) -> impl Future<Output = Result<Self::Res, Self::Err>> + Send {
        self(state, req, service)
    }
}

/// 将给定的异步函数转换为[`Middleware`]。
///
/// # 例子
///
/// ```
/// use boluo_core::handler::handler_fn;
/// use boluo_core::middleware::simple_middleware_fn;
/// use boluo_core::request::Request;
/// use boluo_core::service::{Service, ServiceExt};
///
/// fn assert_service<S, R>(service: S) -> S
/// where
///     S: Service<R>,
/// {
///     service
/// }
///
/// // 日志中间件
/// async fn log<S>(req: Request, service: &S) -> Result<S::Response, S::Error>
/// where
///     S: Service<Request>,
/// {
///     println!("HTTP: {} {}", req.method(), req.uri().path());
///     service.call(req).await
/// }
///
/// let service = handler_fn(|| async {});
/// let service = service.with(simple_middleware_fn(log));
///
/// assert_service(service);
/// ```
pub fn simple_middleware_fn<F>(f: F) -> SimpleMiddlewareFn<F> {
    SimpleMiddlewareFn { f }
}

/// 详情查看[`simple_middleware_fn`]。
#[derive(Clone, Copy)]
pub struct SimpleMiddlewareFn<F> {
    f: F,
}

impl<F, S> Middleware<S> for SimpleMiddlewareFn<F> {
    type Service = SimpleMiddlewareFnService<F, S>;

    fn transform(self, service: S) -> Self::Service {
        SimpleMiddlewareFnService { f: self.f, service }
    }
}

impl<F> std::fmt::Debug for SimpleMiddlewareFn<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimpleMiddlewareFn")
            .field("f", &std::any::type_name::<F>())
            .finish()
    }
}

/// 中间件[`SimpleMiddlewareFn`]返回的服务。
#[derive(Clone, Copy)]
pub struct SimpleMiddlewareFnService<F, S> {
    f: F,
    service: S,
}

impl<F, S, Req, Res, Err> Service<Req> for SimpleMiddlewareFnService<F, S>
where
    for<'a> F: SimpleMiddleware<'a, S, Req, Res = Res, Err = Err>,
    Self: Send + Sync,
{
    type Response = Res;
    type Error = Err;

    fn call(&self, req: Req) -> impl Future<Output = Result<Self::Response, Self::Error>> + Send {
        self.f.call(req, &self.service)
    }
}

impl<F, S> std::fmt::Debug for SimpleMiddlewareFnService<F, S>
where
    S: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimpleMiddlewareFnService")
            .field("f", &std::any::type_name::<F>())
            .field("service", &self.service)
            .finish()
    }
}

trait SimpleMiddleware<'a, S, R>
where
    S: ?Sized,
{
    type Res;
    type Err;

    fn call(
        &self,
        req: R,
        service: &'a S,
    ) -> impl Future<Output = Result<Self::Res, Self::Err>> + Send;
}

impl<'a, S, F, Fut, Req, Res, Err> SimpleMiddleware<'a, S, Req> for F
where
    S: ?Sized + 'a,
    F: Fn(Req, &'a S) -> Fut,
    Fut: Future<Output = Result<Res, Err>> + Send + 'a,
{
    type Res = Res;
    type Err = Err;

    fn call(
        &self,
        req: Req,
        service: &'a S,
    ) -> impl Future<Output = Result<Self::Res, Self::Err>> + Send {
        self(req, service)
    }
}
