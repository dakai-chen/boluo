use std::convert::TryFrom;

use http::header::{HeaderMap, HeaderName, HeaderValue};
use http::method::Method;
use http::version::Version;
use http::{Extensions, Result, Uri};
use sync_wrapper::SyncWrapper;

use crate::body::Body;

#[derive(Default)]
pub struct Request<T = Body> {
    head: RequestParts,
    body: SyncWrapper<T>,
}

#[derive(Default, Clone)]
pub struct RequestParts {
    pub method: Method,
    pub uri: Uri,
    pub version: Version,
    pub headers: HeaderMap<HeaderValue>,
    pub extensions: Extensions,
}

#[derive(Debug)]
pub struct RequestBuilder {
    inner: Result<Request<()>>,
}

impl Request<()> {
    #[inline]
    pub fn builder() -> RequestBuilder {
        RequestBuilder::new()
    }
}

impl<T> Request<T> {
    #[inline]
    pub fn new(body: T) -> Request<T> {
        Request {
            head: RequestParts::new(),
            body: SyncWrapper::new(body),
        }
    }

    #[inline]
    pub fn from_parts(parts: RequestParts, body: T) -> Request<T> {
        Request {
            head: parts,
            body: SyncWrapper::new(body),
        }
    }

    #[inline]
    pub fn method(&self) -> &Method {
        &self.head.method
    }

    #[inline]
    pub fn method_mut(&mut self) -> &mut Method {
        &mut self.head.method
    }

    #[inline]
    pub fn uri(&self) -> &Uri {
        &self.head.uri
    }

    #[inline]
    pub fn uri_mut(&mut self) -> &mut Uri {
        &mut self.head.uri
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
    pub fn parts(&self) -> &RequestParts {
        &self.head
    }

    #[inline]
    pub fn parts_mut(&mut self) -> &mut RequestParts {
        &mut self.head
    }

    #[inline]
    pub fn into_parts(self) -> RequestParts {
        self.head
    }

    #[inline]
    pub fn into_inner(self) -> (RequestParts, T) {
        (self.head, self.body.into_inner())
    }

    #[inline]
    pub fn map<F, U>(self, f: F) -> Request<U>
    where
        F: FnOnce(T) -> U,
    {
        Request {
            body: SyncWrapper::new(f(self.body.into_inner())),
            head: self.head,
        }
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for Request<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Request")
            .field("method", self.method())
            .field("uri", self.uri())
            .field("version", &self.version())
            .field("headers", self.headers())
            .field("body", &std::any::type_name::<T>())
            .finish()
    }
}

impl RequestParts {
    fn new() -> RequestParts {
        RequestParts {
            method: Method::default(),
            uri: Uri::default(),
            version: Version::default(),
            headers: HeaderMap::default(),
            extensions: Extensions::default(),
        }
    }
}

impl std::fmt::Debug for RequestParts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RequestParts")
            .field("method", &self.method)
            .field("uri", &self.uri)
            .field("version", &self.version)
            .field("headers", &self.headers)
            .finish()
    }
}

impl RequestBuilder {
    #[inline]
    pub fn new() -> RequestBuilder {
        RequestBuilder::default()
    }

    pub fn method<T>(self, method: T) -> RequestBuilder
    where
        Method: TryFrom<T>,
        <Method as TryFrom<T>>::Error: Into<http::Error>,
    {
        self.and_then(move |mut req| {
            let method = TryFrom::try_from(method).map_err(Into::into)?;
            req.head.method = method;
            Ok(req)
        })
    }

    pub fn method_ref(&self) -> Option<&Method> {
        self.inner.as_ref().ok().map(|req| &req.head.method)
    }

    pub fn method_mut(&mut self) -> Option<&mut Method> {
        self.inner.as_mut().ok().map(|req| &mut req.head.method)
    }

    pub fn uri<T>(self, uri: T) -> RequestBuilder
    where
        Uri: TryFrom<T>,
        <Uri as TryFrom<T>>::Error: Into<http::Error>,
    {
        self.and_then(move |mut req| {
            let uri = TryFrom::try_from(uri).map_err(Into::into)?;
            req.head.uri = uri;
            Ok(req)
        })
    }

    pub fn uri_ref(&self) -> Option<&Uri> {
        self.inner.as_ref().ok().map(|req| &req.head.uri)
    }

    pub fn uri_mut(&mut self) -> Option<&mut Uri> {
        self.inner.as_mut().ok().map(|req| &mut req.head.uri)
    }

    pub fn version(self, version: Version) -> RequestBuilder {
        self.and_then(move |mut req| {
            req.head.version = version;
            Ok(req)
        })
    }

    pub fn version_ref(&self) -> Option<&Version> {
        self.inner.as_ref().ok().map(|req| &req.head.version)
    }

    pub fn version_mut(&mut self) -> Option<&mut Version> {
        self.inner.as_mut().ok().map(|req| &mut req.head.version)
    }

    pub fn header<K, V>(self, key: K, value: V) -> RequestBuilder
    where
        HeaderName: TryFrom<K>,
        <HeaderName as TryFrom<K>>::Error: Into<http::Error>,
        HeaderValue: TryFrom<V>,
        <HeaderValue as TryFrom<V>>::Error: Into<http::Error>,
    {
        self.and_then(move |mut req| {
            let name = <HeaderName as TryFrom<K>>::try_from(key).map_err(Into::into)?;
            let value = <HeaderValue as TryFrom<V>>::try_from(value).map_err(Into::into)?;
            req.head.headers.append(name, value);
            Ok(req)
        })
    }

    pub fn headers_ref(&self) -> Option<&HeaderMap<HeaderValue>> {
        self.inner.as_ref().ok().map(|req| &req.head.headers)
    }

    pub fn headers_mut(&mut self) -> Option<&mut HeaderMap<HeaderValue>> {
        self.inner.as_mut().ok().map(|req| &mut req.head.headers)
    }

    pub fn extension<T>(self, extension: T) -> RequestBuilder
    where
        T: Clone + Send + Sync + 'static,
    {
        self.and_then(move |mut req| {
            req.head.extensions.insert(extension);
            Ok(req)
        })
    }

    pub fn extensions_ref(&self) -> Option<&Extensions> {
        self.inner.as_ref().ok().map(|req| &req.head.extensions)
    }

    pub fn extensions_mut(&mut self) -> Option<&mut Extensions> {
        self.inner.as_mut().ok().map(|req| &mut req.head.extensions)
    }

    pub fn body<T>(self, body: T) -> Result<Request<T>> {
        self.inner.map(move |req| req.map(|_| body))
    }

    fn and_then<F>(self, func: F) -> Self
    where
        F: FnOnce(Request<()>) -> Result<Request<()>>,
    {
        RequestBuilder {
            inner: self.inner.and_then(func),
        }
    }
}

impl Default for RequestBuilder {
    #[inline]
    fn default() -> RequestBuilder {
        RequestBuilder {
            inner: Ok(Request::new(())),
        }
    }
}
