use crate::middleware::Middleware;
use crate::service::Service;

/// 将给定的异步函数转换为 [`Middleware`]，并可以携带状态。
///
/// # 例子
///
/// ```
/// use boluo_core::handler::handler_fn;
/// use boluo_core::middleware::filter_fn_with_state;
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
/// let service = service.with(filter_fn_with_state("HTTP", log));
/// ```
pub fn filter_fn_with_state<T, F>(state: T, f: F) -> FilterFnWithState<T, F> {
    FilterFnWithState { state, f }
}

/// 详情查看 [`filter_fn_with_state`]。
#[derive(Clone, Copy)]
pub struct FilterFnWithState<T, F> {
    state: T,
    f: F,
}

impl<T, F, S> Middleware<S> for FilterFnWithState<T, F> {
    type Service = FilterFnWithStateService<T, F, S>;

    fn transform(self, service: S) -> Self::Service {
        FilterFnWithStateService {
            state: self.state,
            f: self.f,
            service,
        }
    }
}

impl<T, F> std::fmt::Debug for FilterFnWithState<T, F>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FilterFnWithState")
            .field("state", &self.state)
            .field("f", &std::any::type_name::<F>())
            .finish()
    }
}

/// 中间件 [`FilterFnWithState`] 返回的服务。
#[derive(Clone, Copy)]
pub struct FilterFnWithStateService<T, F, S> {
    state: T,
    f: F,
    service: S,
}

impl<T, F, S, Req, Res, Err> Service<Req> for FilterFnWithStateService<T, F, S>
where
    for<'a> F: FilterWithState<'a, T, S, Req, Res = Res, Err = Err>,
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

impl<T, F, S> std::fmt::Debug for FilterFnWithStateService<T, F, S>
where
    T: std::fmt::Debug,
    S: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FilterFnWithStateService")
            .field("state", &self.state)
            .field("f", &std::any::type_name::<F>())
            .field("service", &self.service)
            .finish()
    }
}

trait FilterWithState<'a, T, S, R>
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

impl<'a, T, S, F, Fut, Req, Res, Err> FilterWithState<'a, T, S, Req> for F
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
/// use boluo_core::middleware::filter_fn;
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
/// let service = service.with(filter_fn(log));
/// ```
pub fn filter_fn<F>(f: F) -> FilterFn<F> {
    FilterFn { f }
}

/// 详情查看 [`filter_fn`]。
#[derive(Clone, Copy)]
pub struct FilterFn<F> {
    f: F,
}

impl<F, S> Middleware<S> for FilterFn<F> {
    type Service = FilterFnService<F, S>;

    fn transform(self, service: S) -> Self::Service {
        FilterFnService { f: self.f, service }
    }
}

impl<F> std::fmt::Debug for FilterFn<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FilterFn")
            .field("f", &std::any::type_name::<F>())
            .finish()
    }
}

/// 中间件 [`FilterFn`] 返回的服务。
#[derive(Clone, Copy)]
pub struct FilterFnService<F, S> {
    f: F,
    service: S,
}

impl<F, S, Req, Res, Err> Service<Req> for FilterFnService<F, S>
where
    for<'a> F: Filter<'a, S, Req, Res = Res, Err = Err>,
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

impl<F, S> std::fmt::Debug for FilterFnService<F, S>
where
    S: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FilterFnService")
            .field("f", &std::any::type_name::<F>())
            .field("service", &self.service)
            .finish()
    }
}

trait Filter<'a, S, R>
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

impl<'a, S, F, Fut, Req, Res, Err> Filter<'a, S, Req> for F
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
