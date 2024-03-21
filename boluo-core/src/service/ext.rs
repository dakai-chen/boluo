use crate::middleware::Middleware;

use super::{
    AndThen, ArcService, BoxCloneService, BoxService, MapErr, MapRequest, MapResponse, MapResult,
    OrElse, Service, Then,
};

/// [`Service`]的扩展特征，提供了一些方便的功能。
pub trait ServiceExt<Req>: Service<Req> {
    /// 在此服务上应用中间件。
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
    fn with<T>(self, middleware: T) -> T::Service
    where
        Self: Sized,
        T: Middleware<Self>,
    {
        middleware.transform(self)
    }

    /// 在此服务执行完成后执行给定的异步函数。
    ///
    /// # 例子
    ///
    /// ```
    /// use boluo_core::service::{service_fn, ServiceExt};
    ///
    /// #[derive(Debug)]
    /// struct MyError;
    ///
    /// impl std::fmt::Display for MyError {
    ///     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    ///         write!(f, "some error message")
    ///     }
    /// }
    ///
    /// impl std::error::Error for MyError {}
    ///
    /// async fn throw_error(_: ()) -> Result<(), MyError> {
    ///     Err(MyError)
    /// }
    ///
    /// let service = service_fn(throw_error);
    /// let service = service.then(|result: Result<(), MyError>| async move {
    ///     if let Err(err) = &result {
    ///         // 打印错误信息。
    ///         println!("{err}");
    ///     }
    ///     result
    /// });
    /// ```
    fn then<F>(self, f: F) -> Then<Self, F>
    where
        Self: Sized,
    {
        Then::new(self, f)
    }

    /// 在此服务执行成功后执行给定的异步函数。
    ///
    /// # 例子
    ///
    /// ```
    /// use boluo_core::body::Body;
    /// use boluo_core::handler::handler_fn;
    /// use boluo_core::response::Response;
    /// use boluo_core::service::ServiceExt;
    /// use boluo_core::BoxError;
    ///
    /// async fn hello() -> &'static str {
    ///     "Hello, World!"
    /// }
    ///
    /// let service = handler_fn(hello);
    /// let service = service.and_then(|response: Response| async move {
    ///     // 清空响应主体。
    ///     Ok::<_, BoxError>(response.map(|_| Body::empty()))
    /// });
    /// ```
    fn and_then<F>(self, f: F) -> AndThen<Self, F>
    where
        Self: Sized,
    {
        AndThen::new(self, f)
    }

    /// 在此服务执行失败后执行给定的异步函数。
    ///
    /// # 例子
    ///
    /// ```
    /// use boluo_core::handler::handler_fn;
    /// use boluo_core::http::StatusCode;
    /// use boluo_core::response::IntoResponse;
    /// use boluo_core::service::ServiceExt;
    /// use boluo_core::BoxError;
    ///
    /// #[derive(Debug)]
    /// struct MyError;
    ///
    /// impl std::fmt::Display for MyError {
    ///     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    ///         write!(f, "some error message")
    ///     }
    /// }
    ///
    /// impl std::error::Error for MyError {}
    ///
    /// async fn throw_error() -> Result<(), MyError> {
    ///     Err(MyError)
    /// }
    ///
    /// let service = handler_fn(throw_error);
    /// let service = service.or_else(|err: BoxError| async move {
    ///     // 捕获错误并转换为响应。
    ///     if let Some(e) = err.downcast_ref::<MyError>() {
    ///         let status = StatusCode::INTERNAL_SERVER_ERROR;
    ///         return Ok((status, format!("{e}")).into_response()?);
    ///     }
    ///     Err(err)
    /// });
    /// ```
    fn or_else<F>(self, f: F) -> OrElse<Self, F>
    where
        Self: Sized,
    {
        OrElse::new(self, f)
    }

    /// 将此服务返回的结果映射为其他值。
    ///
    /// # 例子
    ///
    /// ```
    /// use std::convert::Infallible;
    ///
    /// use boluo_core::service::{service_fn, ServiceExt};
    ///
    /// #[derive(Debug)]
    /// struct MyError;
    ///
    /// impl std::fmt::Display for MyError {
    ///     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    ///         write!(f, "some error message")
    ///     }
    /// }
    ///
    /// impl std::error::Error for MyError {}
    ///
    /// async fn throw_error(_: ()) -> Result<(), MyError> {
    ///     Err(MyError)
    /// }
    ///
    /// let service = service_fn(throw_error);
    /// let service = service.map_result(
    ///     |result: Result<(), MyError>| -> Result<String, Infallible> {
    ///         match result {
    ///             Ok(_) => Ok(format!("")),
    ///             Err(e) => Ok(format!("{e}")),
    ///         }
    ///     },
    /// );
    /// ```
    fn map_result<F>(self, f: F) -> MapResult<Self, F>
    where
        Self: Sized,
    {
        MapResult::new(self, f)
    }

    /// 将此服务返回的响应映射为其他值。
    ///
    /// # 例子
    ///
    /// ```
    /// use std::convert::Infallible;
    ///
    /// use boluo_core::body::Body;
    /// use boluo_core::service::{service_fn, ServiceExt};
    ///
    /// async fn hello(_: ()) -> Result<&'static str, Infallible> {
    ///     Ok("Hello, World!")
    /// }
    ///
    /// let service = service_fn(hello);
    /// let service = service.map_response(|text: &'static str| Body::from(text));
    /// ```
    fn map_response<F>(self, f: F) -> MapResponse<Self, F>
    where
        Self: Sized,
    {
        MapResponse::new(self, f)
    }

    /// 将此服务返回的错误映射为其他值。
    ///
    /// # 例子
    ///
    /// ```
    /// use boluo_core::service::{service_fn, ServiceExt};
    /// use boluo_core::BoxError;
    ///
    /// #[derive(Debug)]
    /// struct MyError;
    ///
    /// impl std::fmt::Display for MyError {
    ///     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    ///         write!(f, "some error message")
    ///     }
    /// }
    ///
    /// impl std::error::Error for MyError {}
    ///
    /// async fn throw_error(_: ()) -> Result<(), MyError> {
    ///     Err(MyError)
    /// }
    ///
    /// let service = service_fn(throw_error);
    /// let service = service.map_err(|err: MyError| BoxError::from(err));
    /// ```
    fn map_err<F>(self, f: F) -> MapErr<Self, F>
    where
        Self: Sized,
    {
        MapErr::new(self, f)
    }

    /// 将发送给此服务的请求映射为其他值。
    ///
    /// # 例子
    ///
    /// ```
    /// use std::convert::Infallible;
    ///
    /// use boluo_core::service::{service_fn, Service, ServiceExt};
    ///
    /// async fn echo(text: String) -> Result<String, Infallible> {
    ///     Ok(text)
    /// }
    ///
    /// let service = service_fn(echo);
    /// let service = service.map_request(|req: &[u8]| {
    ///     // 将字节片转换为包含无效字符的字符串。
    ///     String::from_utf8_lossy(&req).into_owned()
    /// });
    ///
    /// let fut = service.call(b"Hello, World");
    /// ```
    fn map_request<F>(self, f: F) -> MapRequest<Self, F>
    where
        Self: Sized,
    {
        MapRequest::new(self, f)
    }

    /// 将此服务转换为[`Service`]特征对象并装箱。
    ///
    /// 更多详细信息，请参阅[`BoxService`]。
    fn boxed(self) -> BoxService<Req, Self::Response, Self::Error>
    where
        Self: Sized + Send + Sync + 'static,
    {
        BoxService::new(self)
    }

    /// 将此服务转换为[`Service`]特征对象并装箱。
    ///
    /// 更多详细信息，请参阅[`BoxCloneService`]。
    fn boxed_clone(self) -> BoxCloneService<Req, Self::Response, Self::Error>
    where
        Self: Sized + Clone + Send + Sync + 'static,
    {
        BoxCloneService::new(self)
    }

    /// 将此服务转换为[`Service`]特征对象并装箱。
    ///
    /// 更多详细信息，请参阅[`ArcService`]。
    fn boxed_arc(self) -> ArcService<Req, Self::Response, Self::Error>
    where
        Self: Sized + Send + Sync + 'static,
    {
        ArcService::new(self)
    }
}

impl<S, Req> ServiceExt<Req> for S where S: Service<Req> {}
