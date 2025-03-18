//! 中间件的特征和相关类型的定义。

mod middleware_fn;
mod simple;

pub use middleware_fn::*;
pub use simple::*;

/// 用于表示中间件的特征。
///
/// # 例子
///
/// ```
/// use boluo_core::handler::handler_fn;
/// use boluo_core::middleware::Middleware;
/// use boluo_core::request::Request;
/// use boluo_core::response::{IntoResponse, Response};
/// use boluo_core::service::{Service, ServiceExt};
/// use boluo_core::BoxError;
///
/// /// 记录请求日志的中间件。
/// #[derive(Debug, Clone, Copy)]
/// struct Log;
///
/// /// 为日志中间件实现[`Middleware`]特征。
/// impl<S> Middleware<S> for Log
/// where
///     S: Service<Request>,
///     S::Response: IntoResponse,
///     S::Error: Into<BoxError>,
/// {
///     type Service = LogService<S>;
///
///     fn transform(self, service: S) -> Self::Service {
///         LogService { service }
///     }
/// }
///
/// /// 日志中间件生成的新服务。
/// #[derive(Debug, Clone, Copy)]
/// struct LogService<S> {
///     service: S,
/// }
///
/// impl<S> Service<Request> for LogService<S>
/// where
///     S: Service<Request>,
///     S::Response: IntoResponse,
///     S::Error: Into<BoxError>,
/// {
///     type Response = Response;
///     type Error = BoxError;
///
///     async fn call(&self, req: Request) -> Result<Self::Response, Self::Error> {
///         println!("req -> {} {}", req.method(), req.uri().path());
///
///         let result = self
///             .service
///             .call(req)
///             .await
///             .map_err(Into::into)
///             .and_then(|r| r.into_response().map_err(Into::into));
///
///         match &result {
///             Ok(response) => {
///                 println!("res -> {}", response.status());
///             }
///             Err(err) => {
///                 println!("err -> {err}");
///             }
///         }
///
///         result
///     }
/// }
///
/// async fn hello() -> &'static str {
///     "Hello, World!"
/// }
///
/// let service = handler_fn(hello);
/// let service = service.with(Log); // 添加日志中间件。
/// ```
pub trait Middleware<S> {
    /// 新的[`Service`]类型。
    ///
    /// [`Service`]: crate::service::Service
    type Service;

    /// 将给定的[`Service`]对象转换为一个新的[`Service`]。
    ///
    /// [`Service`]: crate::service::Service
    fn transform(self, service: S) -> Self::Service;
}
