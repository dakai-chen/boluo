mod into_response;
pub use into_response::{IntoHeaderError, IntoResponse, IntoResponseParts};

use std::convert::TryFrom;

use http::header::{HeaderMap, HeaderName, HeaderValue};
use http::status::StatusCode;
use http::version::Version;
use http::{Extensions, Result};
use sync_wrapper::SyncWrapper;

use crate::body::Body;

#[derive(Default)]
pub struct Response<T = Body> {
    head: ResponseParts,
    body: SyncWrapper<T>,
}

#[derive(Default, Clone)]
pub struct ResponseParts {
    pub status: StatusCode,
    pub version: Version,
    pub headers: HeaderMap<HeaderValue>,
    pub extensions: Extensions,
}

#[derive(Debug)]
pub struct ResponseBuilder {
    inner: Result<Response<()>>,
}

impl Response<()> {
    #[inline]
    pub fn builder() -> ResponseBuilder {
        ResponseBuilder::new()
    }
}

impl<T> Response<T> {
    #[inline]
    pub fn new(body: T) -> Response<T> {
        Response {
            head: ResponseParts::new(),
            body: SyncWrapper::new(body),
        }
    }

    #[inline]
    pub fn from_parts(parts: ResponseParts, body: T) -> Response<T> {
        Response {
            head: parts,
            body: SyncWrapper::new(body),
        }
    }

    #[inline]
    pub fn status(&self) -> StatusCode {
        self.head.status
    }

    #[inline]
    pub fn status_mut(&mut self) -> &mut StatusCode {
        &mut self.head.status
    }

    #[inline]
    pub fn version(&self) -> Version {
        self.head.version
    }

    #[inline]
    pub fn version_mut(&mut self) -> &mut Version {
        &mut self.head.version
    }

    #[inline]
    pub fn headers(&self) -> &HeaderMap<HeaderValue> {
        &self.head.headers
    }

    #[inline]
    pub fn headers_mut(&mut self) -> &mut HeaderMap<HeaderValue> {
        &mut self.head.headers
    }

    #[inline]
    pub fn extensions(&self) -> &Extensions {
        &self.head.extensions
    }

    #[inline]
    pub fn extensions_mut(&mut self) -> &mut Extensions {
        &mut self.head.extensions
    }

    #[inline]
    pub fn body_mut(&mut self) -> &mut T {
        self.body.get_mut()
    }

    #[inline]
    pub fn into_body(self) -> T {
        self.body.into_inner()
    }

    #[inline]
    pub fn parts(&self) -> &ResponseParts {
        &self.head
    }

    #[inline]
    pub fn parts_mut(&mut self) -> &mut ResponseParts {
        &mut self.head
    }

    #[inline]
    pub fn into_parts(self) -> ResponseParts {
        self.head
    }

    #[inline]
    pub fn into_inner(self) -> (ResponseParts, T) {
        (self.head, self.body.into_inner())
    }

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
    #[inline]
    pub fn new() -> ResponseBuilder {
        ResponseBuilder::default()
    }

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

    pub fn status_ref(&self) -> Option<&StatusCode> {
        self.inner.as_ref().ok().map(|res| &res.head.status)
    }

    pub fn status_mut(&mut self) -> Option<&mut StatusCode> {
        self.inner.as_mut().ok().map(|res| &mut res.head.status)
    }

    pub fn version(self, version: Version) -> ResponseBuilder {
        self.and_then(move |mut res| {
            res.head.version = version;
            Ok(res)
        })
    }

    pub fn version_ref(&self) -> Option<&Version> {
        self.inner.as_ref().ok().map(|res| &res.head.version)
    }

    pub fn version_mut(&mut self) -> Option<&mut Version> {
        self.inner.as_mut().ok().map(|res| &mut res.head.version)
    }

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

    pub fn headers_ref(&self) -> Option<&HeaderMap<HeaderValue>> {
        self.inner.as_ref().ok().map(|res| &res.head.headers)
    }

    pub fn headers_mut(&mut self) -> Option<&mut HeaderMap<HeaderValue>> {
        self.inner.as_mut().ok().map(|res| &mut res.head.headers)
    }

    pub fn extension<T>(self, extension: T) -> ResponseBuilder
    where
        T: Clone + Send + Sync + 'static,
    {
        self.and_then(move |mut res| {
            res.head.extensions.insert(extension);
            Ok(res)
        })
    }

    pub fn extensions_ref(&self) -> Option<&Extensions> {
        self.inner.as_ref().ok().map(|res| &res.head.extensions)
    }

    pub fn extensions_mut(&mut self) -> Option<&mut Extensions> {
        self.inner.as_mut().ok().map(|res| &mut res.head.extensions)
    }

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
