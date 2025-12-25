use crate::middleware::Middleware;
use crate::service::Service;

/// 将给定的异步函数转换为 [`Middleware`]，并可以携带状态。
///
/// # 例子
///
/// ```
/// use boluo_core::handler::handler_fn;
/// use boluo_core::middleware::around_with_state_fn;
/// use boluo_core::request::Request;
/// use boluo_core::service::{Service, ServiceExt};
///
/// // 日志中间件
/// async fn log<S>(prefix: &&str, request: Request, service: &S) -> Result<S::Response, S::Error>
/// where
///     S: Service<Request>,
/// {
///     println!("{prefix}: {} {}", request.method(), request.uri().path());
///     service.call(request).await
/// }
///
/// let service = handler_fn(|| async {});
/// let service = service.with(around_with_state_fn("HTTP", log));
/// ```
pub fn around_with_state_fn<T, F>(state: T, f: F) -> AroundWithStateFn<T, F> {
    AroundWithStateFn { state, f }
}

/// 详情查看 [`around_with_state_fn`]。
#[derive(Clone, Copy)]
pub struct AroundWithStateFn<T, F> {
    state: T,
    f: F,
}

impl<T, F, S> Middleware<S> for AroundWithStateFn<T, F> {
    type Service = AroundWithStateFnService<T, F, S>;

    fn transform(self, service: S) -> Self::Service {
        AroundWithStateFnService {
            state: self.state,
            f: self.f,
            service,
        }
    }
}

impl<T, F> std::fmt::Debug for AroundWithStateFn<T, F>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AroundWithStateFn")
            .field("state", &self.state)
            .field("f", &std::any::type_name::<F>())
            .finish()
    }
}

/// 中间件 [`AroundWithStateFn`] 返回的服务。
#[derive(Clone, Copy)]
pub struct AroundWithStateFnService<T, F, S> {
    state: T,
    f: F,
    service: S,
}

impl<T, F, S, Req, Res, Err> Service<Req> for AroundWithStateFnService<T, F, S>
where
    for<'a> F: AroundWithState<'a, T, S, Req, Res = Res, Err = Err>,
    Self: Send + Sync,
{
    type Response = Res;
    type Error = Err;

    fn call(
        &self,
        request: Req,
    ) -> impl Future<Output = Result<Self::Response, Self::Error>> + Send {
        self.f.call(&self.state, request, &self.service)
    }
}

impl<T, F, S> std::fmt::Debug for AroundWithStateFnService<T, F, S>
where
    T: std::fmt::Debug,
    S: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AroundWithStateFnService")
            .field("state", &self.state)
            .field("f", &std::any::type_name::<F>())
            .field("service", &self.service)
            .finish()
    }
}

trait AroundWithState<'a, T, S, R>
where
    T: ?Sized,
    S: ?Sized,
{
    type Res;
    type Err;

    fn call(
        &self,
        state: &'a T,
        request: R,
        service: &'a S,
    ) -> impl Future<Output = Result<Self::Res, Self::Err>> + Send;
}

impl<'a, T, S, F, Fut, Req, Res, Err> AroundWithState<'a, T, S, Req> for F
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
        request: Req,
        service: &'a S,
    ) -> impl Future<Output = Result<Self::Res, Self::Err>> + Send {
        self(state, request, service)
    }
}

/// 将给定的异步函数转换为 [`Middleware`]。
///
/// # 例子
///
/// ```
/// use boluo_core::handler::handler_fn;
/// use boluo_core::middleware::around_fn;
/// use boluo_core::request::Request;
/// use boluo_core::service::{Service, ServiceExt};
///
/// // 日志中间件
/// async fn log<S>(request: Request, service: &S) -> Result<S::Response, S::Error>
/// where
///     S: Service<Request>,
/// {
///     println!("HTTP: {} {}", request.method(), request.uri().path());
///     service.call(request).await
/// }
///
/// let service = handler_fn(|| async {});
/// let service = service.with(around_fn(log));
/// ```
pub fn around_fn<F>(f: F) -> AroundFn<F> {
    AroundFn { f }
}

/// 详情查看 [`around_fn`]。
#[derive(Clone, Copy)]
pub struct AroundFn<F> {
    f: F,
}

impl<F, S> Middleware<S> for AroundFn<F> {
    type Service = AroundFnService<F, S>;

    fn transform(self, service: S) -> Self::Service {
        AroundFnService { f: self.f, service }
    }
}

impl<F> std::fmt::Debug for AroundFn<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AroundFn")
            .field("f", &std::any::type_name::<F>())
            .finish()
    }
}

/// 中间件 [`AroundFn`] 返回的服务。
#[derive(Clone, Copy)]
pub struct AroundFnService<F, S> {
    f: F,
    service: S,
}

impl<F, S, Req, Res, Err> Service<Req> for AroundFnService<F, S>
where
    for<'a> F: Around<'a, S, Req, Res = Res, Err = Err>,
    Self: Send + Sync,
{
    type Response = Res;
    type Error = Err;

    fn call(
        &self,
        request: Req,
    ) -> impl Future<Output = Result<Self::Response, Self::Error>> + Send {
        self.f.call(request, &self.service)
    }
}

impl<F, S> std::fmt::Debug for AroundFnService<F, S>
where
    S: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AroundFnService")
            .field("f", &std::any::type_name::<F>())
            .field("service", &self.service)
            .finish()
    }
}

trait Around<'a, S, R>
where
    S: ?Sized,
{
    type Res;
    type Err;

    fn call(
        &self,
        request: R,
        service: &'a S,
    ) -> impl Future<Output = Result<Self::Res, Self::Err>> + Send;
}

impl<'a, S, F, Fut, Req, Res, Err> Around<'a, S, Req> for F
where
    S: ?Sized + 'a,
    F: Fn(Req, &'a S) -> Fut,
    Fut: Future<Output = Result<Res, Err>> + Send + 'a,
{
    type Res = Res;
    type Err = Err;

    fn call(
        &self,
        request: Req,
        service: &'a S,
    ) -> impl Future<Output = Result<Self::Res, Self::Err>> + Send {
        self(request, service)
    }
}
