//! HTTP响应。

mod into_response;
pub use into_response::{HeaderResponseError, IntoResponse, IntoResponseParts};

use std::convert::TryFrom;

use http::header::{HeaderMap, HeaderName, HeaderValue};
use http::status::StatusCode;
use http::version::Version;
use http::{Extensions, Result};
use sync_wrapper::SyncWrapper;

use crate::body::Body;

/// HTTP响应。
///
/// HTTP响应由头部和可选的主体组成。主体是泛型的，允许任意类型来表示HTTP响应的主体。
#[derive(Default)]
pub struct Response<T = Body> {
    head: ResponseParts,
    body: SyncWrapper<T>,
}

/// HTTP响应的头部。
///
/// HTTP响应的头部由状态码、版本、一组标头和扩展组成。
#[derive(Default, Clone)]
pub struct ResponseParts {
    /// HTTP响应的状态码。
    pub status: StatusCode,
    /// HTTP响应的版本。
    pub version: Version,
    /// HTTP响应的标头集。
    pub headers: HeaderMap<HeaderValue>,
    /// HTTP响应的扩展。
    pub extensions: Extensions,
}

/// HTTP响应的构建器。
#[derive(Debug)]
pub struct ResponseBuilder {
    inner: Result<Response<()>>,
}

impl Response<()> {
    /// 创建新的构建器以构建响应。
    ///
    /// # 例子
    ///
    /// ```
    /// use boluo_core::response::Response;
    ///
    /// let response = Response::builder()
    ///     .status(200)
    ///     .header("X-Custom-Foo", "Bar")
    ///     .body(())
    ///     .unwrap();
    /// ```
    #[inline]
    pub fn builder() -> ResponseBuilder {
        ResponseBuilder::new()
    }
}

impl<T> Response<T> {
    /// 使用给定的主体创建一个空白的响应。
    ///
    /// 此响应的头部将被设置为默认值。
    ///
    /// # 例子
    ///
    /// ```
    /// use boluo_core::http::StatusCode;
    /// use boluo_core::response::Response;
    ///
    /// let mut response = Response::new("hello world");
    ///
    /// assert_eq!(response.status(), StatusCode::OK);
    /// assert_eq!(*response.body_mut(), "hello world");
    /// ```
    #[inline]
    pub fn new(body: T) -> Response<T> {
        Response {
            head: ResponseParts::new(),
            body: SyncWrapper::new(body),
        }
    }

    /// 使用给定的头部和主体创建响应。
    ///
    /// # 例子
    ///
    /// ```
    /// use boluo_core::http::StatusCode;
    /// use boluo_core::response::Response;
    ///
    /// let response = Response::new("hello world");
    /// let (mut parts, body) = response.into_inner();
    ///
    /// parts.status = StatusCode::BAD_REQUEST;
    /// let mut response = Response::from_parts(parts, body);
    ///
    /// assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    /// assert_eq!(*response.body_mut(), "hello world");
    /// ```
    #[inline]
    pub fn from_parts(parts: ResponseParts, body: T) -> Response<T> {
        Response {
            head: parts,
            body: SyncWrapper::new(body),
        }
    }

    /// 获取响应的状态码。
    ///
    /// # 例子
    ///
    /// ```
    /// use boluo_core::http::StatusCode;
    /// use boluo_core::response::Response;
    ///
    /// let response: Response<()> = Response::default();
    ///
    /// assert_eq!(response.status(), StatusCode::OK);
    /// ```
    #[inline]
    pub fn status(&self) -> StatusCode {
        self.head.status
    }

    /// 获取响应的状态码的可变引用。
    ///
    /// # 例子
    ///
    /// ```
    /// use boluo_core::http::StatusCode;
    /// use boluo_core::response::Response;
    ///
    /// let mut response: Response<()> = Response::default();
    /// *response.status_mut() = StatusCode::CREATED;
    ///
    /// assert_eq!(response.status(), StatusCode::CREATED);
    /// ```
    #[inline]
    pub fn status_mut(&mut self) -> &mut StatusCode {
        &mut self.head.status
    }

    /// 获取响应的HTTP版本。
    ///
    /// # 例子
    ///
    /// ```
    /// use boluo_core::http::Version;
    /// use boluo_core::response::Response;
    ///
    /// let response: Response<()> = Response::default();
    ///
    /// assert_eq!(response.version(), Version::HTTP_11);
    /// ```
    #[inline]
    pub fn version(&self) -> Version {
        self.head.version
    }

    /// 获取响应的HTTP版本的可变引用。
    ///
    /// # 例子
    ///
    /// ```
    /// use boluo_core::http::Version;
    /// use boluo_core::response::Response;
    ///
    /// let mut response: Response<()> = Response::default();
    /// *response.version_mut() = Version::HTTP_2;
    ///
    /// assert_eq!(response.version(), Version::HTTP_2);
    /// ```
    #[inline]
    pub fn version_mut(&mut self) -> &mut Version {
        &mut self.head.version
    }

    /// 获取响应的标头集的引用。
    ///
    /// # 例子
    ///
    /// ```
    /// use boluo_core::response::Response;
    ///
    /// let response: Response<()> = Response::default();
    ///
    /// assert!(response.headers().is_empty());
    /// ```
    #[inline]
    pub fn headers(&self) -> &HeaderMap<HeaderValue> {
        &self.head.headers
    }

    /// 获取响应的标头集的可变引用。
    ///
    /// # 例子
    ///
    /// ```
    /// use boluo_core::http::header::*;
    /// use boluo_core::response::Response;
    ///
    /// let mut response: Response<()> = Response::default();
    /// response.headers_mut().insert(HOST, HeaderValue::from_static("world"));
    ///
    /// assert!(!response.headers().is_empty());
    /// ```
    #[inline]
    pub fn headers_mut(&mut self) -> &mut HeaderMap<HeaderValue> {
        &mut self.head.headers
    }

    /// 获取响应的扩展的引用。
    ///
    /// # 例子
    ///
    /// ```
    /// use boluo_core::response::Response;
    ///
    /// let response: Response<()> = Response::default();
    ///
    /// assert!(response.extensions().get::<i32>().is_none());
    /// ```
    #[inline]
    pub fn extensions(&self) -> &Extensions {
        &self.head.extensions
    }

    /// 获取响应的扩展的可变引用。
    ///
    /// # 例子
    ///
    /// ```
    /// use boluo_core::response::Response;
    ///
    /// let mut response: Response<()> = Response::default();
    /// response.extensions_mut().insert("hello");
    ///
    /// assert_eq!(response.extensions().get(), Some(&"hello"));
    /// ```
    #[inline]
    pub fn extensions_mut(&mut self) -> &mut Extensions {
        &mut self.head.extensions
    }

    /// 获取响应的主体的可变引用。
    ///
    /// # 例子
    ///
    /// ```
    /// use boluo_core::response::Response;
    ///
    /// let mut response: Response<String> = Response::default();
    /// response.body_mut().push_str("hello world");
    ///
    /// assert!(!response.body_mut().is_empty());
    /// ```
    #[inline]
    pub fn body_mut(&mut self) -> &mut T {
        self.body.get_mut()
    }

    /// 消耗响应，返回响应的主体。
    ///
    /// # 例子
    ///
    /// ```
    /// use boluo_core::response::Response;
    ///
    /// let response = Response::new(10);
    /// let body = response.into_body();
    ///
    /// assert_eq!(body, 10);
    /// ```
    #[inline]
    pub fn into_body(self) -> T {
        self.body.into_inner()
    }

    /// 获取响应的头部的引用。
    ///
    /// # 例子
    ///
    /// ```
    /// use boluo_core::http::StatusCode;
    /// use boluo_core::response::Response;
    ///
    /// let response: Response<()> = Response::default();
    ///
    /// assert_eq!(response.parts().status, StatusCode::OK);
    /// ```
    #[inline]
    pub fn parts(&self) -> &ResponseParts {
        &self.head
    }

    /// 获取响应的头部的可变引用。
    ///
    /// # 例子
    ///
    /// ```
    /// use boluo_core::http::StatusCode;
    /// use boluo_core::response::Response;
    ///
    /// let mut response: Response<()> = Response::default();
    /// response.parts_mut().status = StatusCode::CREATED;
    ///
    /// assert_eq!(response.status(), StatusCode::CREATED);
    /// ```
    #[inline]
    pub fn parts_mut(&mut self) -> &mut ResponseParts {
        &mut self.head
    }

    /// 消耗响应，返回响应的头部。
    ///
    /// # 例子
    ///
    /// ```
    /// use boluo_core::http::StatusCode;
    /// use boluo_core::response::Response;
    ///
    /// let response = Response::new(());
    /// let parts = response.into_parts();
    ///
    /// assert_eq!(parts.status, StatusCode::OK);
    /// ```
    #[inline]
    pub fn into_parts(self) -> ResponseParts {
        self.head
    }

    /// 消耗响应，返回响应的头部和主体。
    ///
    /// # 例子
    ///
    /// ```
    /// use boluo_core::http::StatusCode;
    /// use boluo_core::response::Response;
    ///
    /// let response: Response<()> = Response::default();
    /// let (parts, body) = response.into_inner();
    ///
    /// assert_eq!(parts.status, StatusCode::OK);
    /// ```
    #[inline]
    pub fn into_inner(self) -> (ResponseParts, T) {
        (self.head, self.body.into_inner())
    }

    /// 消耗响应，返回带有给定主体的新响应，其主体为传入闭包的返回值。
    ///
    /// # 例子
    ///
    /// ```
    /// use boluo_core::response::Response;
    ///
    /// let response = Response::builder().body("some string").unwrap();
    /// let mut mapped_response: Response<&[u8]> = response.map(|b| {
    ///   assert_eq!(b, "some string");
    ///   b.as_bytes()
    /// });
    ///
    /// assert_eq!(mapped_response.body_mut(), &"some string".as_bytes());
    /// ```
    #[inline]
    pub fn map<F, U>(self, f: F) -> Response<U>
    where
        F: FnOnce(T) -> U,
    {
        Response {
            body: SyncWrapper::new(f(self.body.into_inner())),
            head: self.head,
        }
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for Response<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Response")
            .field("status", &self.status())
            .field("version", &self.version())
            .field("headers", self.headers())
            .field("body", &std::any::type_name::<T>())
            .finish()
    }
}

impl ResponseParts {
    fn new() -> ResponseParts {
        ResponseParts {
            status: StatusCode::default(),
            version: Version::default(),
            headers: HeaderMap::default(),
            extensions: Extensions::default(),
        }
    }
}

impl std::fmt::Debug for ResponseParts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResponseParts")
            .field("status", &self.status)
            .field("version", &self.version)
            .field("headers", &self.headers)
            .finish()
    }
}

impl ResponseBuilder {
    /// 创建构建器的默认实例以构建响应。
    ///
    /// # 例子
    ///
    /// ```
    /// use boluo_core::response::ResponseBuilder;
    ///
    /// let response = ResponseBuilder::new()
    ///     .status(200)
    ///     .body(())
    ///     .unwrap();
    /// ```
    #[inline]
    pub fn new() -> ResponseBuilder {
        ResponseBuilder::default()
    }

    /// 设置响应的状态码。
    ///
    /// 默认情况下，这是`200`。
    ///
    /// # 例子
    ///
    /// ```
    /// use boluo_core::response::Response;
    ///
    /// let response = Response::builder()
    ///     .status(200)
    ///     .body(())
    ///     .unwrap();
    /// ```
    pub fn status<T>(self, status: T) -> ResponseBuilder
    where
        StatusCode: TryFrom<T>,
        <StatusCode as TryFrom<T>>::Error: Into<http::Error>,
    {
        self.and_then(move |mut res| {
            let status = TryFrom::try_from(status).map_err(Into::into)?;
            res.head.status = status;
            Ok(res)
        })
    }

    /// 获取响应的状态码的引用。
    ///
    /// 默认情况下，这是`200`。如果构建器有错误，则返回[`None`]。
    ///
    /// # 例子
    ///
    /// ```
    /// use boluo_core::http::StatusCode;
    /// use boluo_core::response::Response;
    ///
    /// let mut res = Response::builder();
    /// assert_eq!(res.status_ref().unwrap(), &StatusCode::OK);
    ///
    /// res = res.status(StatusCode::BAD_REQUEST);
    /// assert_eq!(res.status_ref().unwrap(), &StatusCode::BAD_REQUEST);
    /// ```
    pub fn status_ref(&self) -> Option<&StatusCode> {
        self.inner.as_ref().ok().map(|res| &res.head.status)
    }

    /// 获取响应的状态码的可变引用。
    ///
    /// 默认情况下，这是`200`。如果构建器有错误，则返回[`None`]。
    ///
    /// # 例子
    ///
    /// ```
    /// use boluo_core::http::StatusCode;
    /// use boluo_core::response::Response;
    ///
    /// let mut res = Response::builder();
    /// assert_eq!(res.status_ref().unwrap(), &StatusCode::OK);
    ///
    /// *res.status_mut().unwrap() = StatusCode::BAD_REQUEST;
    /// assert_eq!(res.status_ref().unwrap(), &StatusCode::BAD_REQUEST);
    /// ```
    pub fn status_mut(&mut self) -> Option<&mut StatusCode> {
        self.inner.as_mut().ok().map(|res| &mut res.head.status)
    }

    /// 设置响应的HTTP版本。
    ///
    /// 默认情况下，这是`HTTP/1.1`。
    ///
    /// # 例子
    ///
    /// ```
    /// use boluo_core::http::Version;
    /// use boluo_core::response::Response;
    ///
    /// let response = Response::builder()
    ///     .version(Version::HTTP_2)
    ///     .body(())
    ///     .unwrap();
    /// ```
    pub fn version(self, version: Version) -> ResponseBuilder {
        self.and_then(move |mut res| {
            res.head.version = version;
            Ok(res)
        })
    }

    /// 获取响应的HTTP版本的引用。
    ///
    /// 默认情况下，这是`HTTP/1.1`。如果构建器有错误，则返回[`None`]。
    ///
    /// # 例子
    ///
    /// ```
    /// use boluo_core::http::Version;
    /// use boluo_core::response::Response;
    ///
    /// let mut res = Response::builder();
    /// assert_eq!(res.version_ref().unwrap(), &Version::HTTP_11);
    ///
    /// res = res.version(Version::HTTP_2);
    /// assert_eq!(res.version_ref().unwrap(), &Version::HTTP_2);
    /// ```
    pub fn version_ref(&self) -> Option<&Version> {
        self.inner.as_ref().ok().map(|res| &res.head.version)
    }

    /// 获取响应的HTTP版本的可变引用。
    ///
    /// 默认情况下，这是`HTTP/1.1`。如果构建器有错误，则返回[`None`]。
    ///
    /// # 例子
    ///
    /// ```
    /// use boluo_core::http::Version;
    /// use boluo_core::response::Response;
    ///
    /// let mut res = Response::builder();
    /// assert_eq!(res.version_ref().unwrap(), &Version::HTTP_11);
    ///
    /// *res.version_mut().unwrap() = Version::HTTP_2;
    /// assert_eq!(res.version_ref().unwrap(), &Version::HTTP_2);
    /// ```
    pub fn version_mut(&mut self) -> Option<&mut Version> {
        self.inner.as_mut().ok().map(|res| &mut res.head.version)
    }

    /// 将标头追加到响应中。
    ///
    /// 此函数将提供的键值对追加到响应内部的[`HeaderMap`]中。本质上，
    /// 这相当于调用[`HeaderMap::append`]。
    ///
    /// # 例子
    ///
    /// ```
    /// use boluo_core::response::Response;
    ///
    /// let response = Response::builder()
    ///     .header("Content-Type", "text/html")
    ///     .header("X-Custom-Foo", "bar")
    ///     .header("Content-Length", 0)
    ///     .body(())
    ///     .unwrap();
    /// ```
    pub fn header<K, V>(self, key: K, value: V) -> ResponseBuilder
    where
        HeaderName: TryFrom<K>,
        <HeaderName as TryFrom<K>>::Error: Into<http::Error>,
        HeaderValue: TryFrom<V>,
        <HeaderValue as TryFrom<V>>::Error: Into<http::Error>,
    {
        self.and_then(move |mut res| {
            let name = <HeaderName as TryFrom<K>>::try_from(key).map_err(Into::into)?;
            let value = <HeaderValue as TryFrom<V>>::try_from(value).map_err(Into::into)?;
            res.head.headers.append(name, value);
            Ok(res)
        })
    }

    /// 获取响应的标头集的引用。
    ///
    /// 如果构建器有错误，则返回[`None`]。
    ///
    /// # 例子
    ///
    /// ```
    /// use boluo_core::response::Response;
    ///
    /// let res = Response::builder()
    ///     .header("Accept", "text/html")
    ///     .header("X-Custom-Foo", "bar");
    ///
    /// let headers = res.headers_ref().unwrap();
    ///
    /// assert_eq!(headers["Accept"], "text/html");
    /// assert_eq!(headers["X-Custom-Foo"], "bar");
    /// ```
    pub fn headers_ref(&self) -> Option<&HeaderMap<HeaderValue>> {
        self.inner.as_ref().ok().map(|res| &res.head.headers)
    }

    /// 获取响应的标头集的可变引用。
    ///
    /// 如果构建器有错误，则返回[`None`]。
    ///
    /// # 例子
    ///
    /// ```
    /// use boluo_core::http::HeaderValue;
    /// use boluo_core::response::Response;
    ///
    /// let mut res = Response::builder();
    ///
    /// let headers = res.headers_mut().unwrap();
    /// headers.insert("Accept", HeaderValue::from_static("text/html"));
    /// headers.insert("X-Custom-Foo", HeaderValue::from_static("bar"));
    ///
    /// let headers = res.headers_ref().unwrap();
    /// assert_eq!( headers["Accept"], "text/html" );
    /// assert_eq!( headers["X-Custom-Foo"], "bar" );
    /// ```
    pub fn headers_mut(&mut self) -> Option<&mut HeaderMap<HeaderValue>> {
        self.inner.as_mut().ok().map(|res| &mut res.head.headers)
    }

    /// 将一个类型添加到响应的扩展中。
    ///
    /// # 例子
    ///
    /// ```
    /// use boluo_core::response::Response;
    ///
    /// let response = Response::builder()
    ///     .extension("My Extension")
    ///     .body(())
    ///     .unwrap();
    ///
    /// assert_eq!(response.extensions().get::<&'static str>(),
    ///            Some(&"My Extension"));
    /// ```
    pub fn extension<T>(self, extension: T) -> ResponseBuilder
    where
        T: Clone + Send + Sync + 'static,
    {
        self.and_then(move |mut res| {
            res.head.extensions.insert(extension);
            Ok(res)
        })
    }

    /// 获取响应的扩展的引用。
    ///
    /// 如果构建器有错误，则返回[`None`]。
    ///
    /// # 例子
    ///
    /// ```
    /// use boluo_core::response::Response;
    ///
    /// let res = Response::builder().extension("My Extension").extension(5u32);
    /// let extensions = res.extensions_ref().unwrap();
    ///
    /// assert_eq!(extensions.get::<&'static str>(), Some(&"My Extension"));
    /// assert_eq!(extensions.get::<u32>(), Some(&5u32));
    /// ```
    pub fn extensions_ref(&self) -> Option<&Extensions> {
        self.inner.as_ref().ok().map(|res| &res.head.extensions)
    }

    /// 获取响应的扩展的可变引用。
    ///
    /// 如果构建器有错误，则返回[`None`]。
    ///
    /// # 例子
    ///
    /// ```
    /// use boluo_core::response::Response;
    ///
    /// let mut res = Response::builder().extension("My Extension");
    /// let mut extensions = res.extensions_mut().unwrap();
    /// assert_eq!(extensions.get::<&'static str>(), Some(&"My Extension"));
    ///
    /// extensions.insert(5u32);
    /// assert_eq!(extensions.get::<u32>(), Some(&5u32));
    /// ```
    pub fn extensions_mut(&mut self) -> Option<&mut Extensions> {
        self.inner.as_mut().ok().map(|res| &mut res.head.extensions)
    }

    /// 消耗构建器，使用给定的主体构建响应。
    ///
    /// # 错误
    ///
    /// 如果之前配置的任意一个参数发生错误，则在调用此函数时将返回错误。
    ///
    /// # 例子
    ///
    /// ```
    /// use boluo_core::response::Response;
    ///
    /// let response = Response::builder()
    ///     .body(())
    ///     .unwrap();
    /// ```
    pub fn body<T>(self, body: T) -> Result<Response<T>> {
        self.inner.map(move |res| res.map(|_| body))
    }

    fn and_then<F>(self, func: F) -> Self
    where
        F: FnOnce(Response<()>) -> Result<Response<()>>,
    {
        ResponseBuilder {
            inner: self.inner.and_then(func),
        }
    }
}

impl Default for ResponseBuilder {
    #[inline]
    fn default() -> ResponseBuilder {
        ResponseBuilder {
            inner: Ok(Response::new(())),
        }
    }
}
