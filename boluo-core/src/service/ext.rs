use crate::middleware::Middleware;
use crate::util::assert_service;

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
    /// let service = service.then(|result| async move {
    ///     if let Err(err) = &result {
    ///         // 打印错误信息。
    ///         println!("{err}");
    ///     }
    ///     result
    /// });
    /// ```
    fn then<F, Fut, Res, Err>(self, f: F) -> Then<Self, F>
    where
        Self: Sized,
        F: Fn(Result<Self::Response, Self::Error>) -> Fut + Send + Sync,
        Fut: Future<Output = Result<Res, Err>> + Send,
    {
        assert_service(Then::new(self, f))
    }

    /// 在此服务执行成功后执行给定的异步函数。
    ///
    /// # 例子
    ///
    /// ```
    /// use boluo_core::body::Body;
    /// use boluo_core::handler::handler_fn;
    /// use boluo_core::service::ServiceExt;
    /// use boluo_core::BoxError;
    ///
    /// async fn hello() -> &'static str {
    ///     "Hello, World!"
    /// }
    ///
    /// let service = handler_fn(hello);
    /// let service = service.and_then(|response| async move {
    ///     // 清空响应主体。
    ///     Ok::<_, BoxError>(response.map(|_| Body::empty()))
    /// });
    /// ```
    fn and_then<F, Fut, Res>(self, f: F) -> AndThen<Self, F>
    where
        Self: Sized,
        F: Fn(Self::Response) -> Fut + Send + Sync,
        Fut: Future<Output = Result<Res, Self::Error>> + Send,
    {
        assert_service(AndThen::new(self, f))
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
    /// let service = service.or_else(|err| async move {
    ///     // 捕获错误并转换为响应。
    ///     if let Some(e) = err.downcast_ref::<MyError>() {
    ///         let status = StatusCode::INTERNAL_SERVER_ERROR;
    ///         return Ok((status, format!("{e}")).into_response()?);
    ///     }
    ///     Err(err)
    /// });
    /// ```
    fn or_else<F, Fut, Err>(self, f: F) -> OrElse<Self, F>
    where
        Self: Sized,
        F: Fn(Self::Error) -> Fut + Send + Sync,
        Fut: Future<Output = Result<Self::Response, Err>> + Send,
    {
        assert_service(OrElse::new(self, f))
    }

    /// 将此服务返回的结果映射为其他值。
    ///
    /// # 例子
    ///
    /// ```
    /// use std::convert::Infallible;
    ///
    /// use boluo_core::response::{IntoResponse, Response};
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
    /// let service = service.map_result(|result| -> Result<Response, Infallible> {
    ///     match result {
    ///         Ok(r) => r.into_response(),
    ///         Err(e) => format!("{e}").into_response(),
    ///     }
    /// });
    /// ```
    fn map_result<F, Res, Err>(self, f: F) -> MapResult<Self, F>
    where
        Self: Sized,
        F: Fn(Result<Self::Response, Self::Error>) -> Result<Res, Err> + Send + Sync,
    {
        assert_service(MapResult::new(self, f))
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
    /// let service = service.map_response(|text| Body::from(text));
    /// ```
    fn map_response<F, Res>(self, f: F) -> MapResponse<Self, F>
    where
        Self: Sized,
        F: Fn(Self::Response) -> Res + Send + Sync,
    {
        assert_service(MapResponse::new(self, f))
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
    /// let service = service.map_err(|err| BoxError::from(err));
    /// ```
    fn map_err<F, Err>(self, f: F) -> MapErr<Self, F>
    where
        Self: Sized,
        F: Fn(Self::Error) -> Err + Send + Sync,
    {
        assert_service(MapErr::new(self, f))
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
    /// let service = service.map_request(|slice: &[u8]| {
    ///     // 将字节片转换为包含无效字符的字符串。
    ///     String::from_utf8_lossy(slice).into_owned()
    /// });
    ///
    /// let fut = service.call(b"Hello, World");
    /// ```
    fn map_request<F, R>(self, f: F) -> MapRequest<Self, F>
    where
        Self: Sized,
        F: Fn(R) -> Req + Send + Sync,
    {
        assert_service(MapRequest::new(self, f))
    }

    /// 将此服务转换为[`Service`]特征对象并装箱。
    ///
    /// 更多详细信息，请参阅[`BoxService`]。
    fn boxed(self) -> BoxService<Req, Self::Response, Self::Error>
    where
        Self: Sized + 'static,
    {
        assert_service(BoxService::new(self))
    }

    /// 将此服务转换为[`Service`]特征对象并装箱。
    ///
    /// 更多详细信息，请参阅[`BoxCloneService`]。
    fn boxed_clone(self) -> BoxCloneService<Req, Self::Response, Self::Error>
    where
        Self: Sized + Clone + 'static,
    {
        assert_service(BoxCloneService::new(self))
    }

    /// 将此服务转换为[`Service`]特征对象并装箱。
    ///
    /// 更多详细信息，请参阅[`ArcService`]。
    fn boxed_arc(self) -> ArcService<Req, Self::Response, Self::Error>
    where
        Self: Sized + 'static,
    {
        assert_service(ArcService::new(self))
    }
}

impl<S: ?Sized, Req> ServiceExt<Req> for S where S: Service<Req> {}
