use std::marker::PhantomData;
use std::str::FromStr;

use boluo_core::extract::{FromRequest, Name};
use boluo_core::http::{HeaderName, HeaderValue};
use boluo_core::request::Request;
use boluo_core::BoxError;

/// 根据标头名获取原始标头值的提取器，不对标头值进行解析。
///
/// # 例子
///
/// ```
/// use boluo::extract::RawHeaderOfName;
///
/// #[boluo::route("/", method = "GET")]
/// async fn handler(RawHeaderOfName(host, _): RawHeaderOfName<name::host>) {
///     // ...
/// }
///
/// mod name {
///     #![allow(non_camel_case_types)]
///     boluo::name! {
///         pub host = "host";
///     }
/// }
/// ```
pub struct RawHeaderOfName<N>(pub HeaderValue, pub PhantomData<fn(N) -> N>);

impl<N> std::ops::Deref for RawHeaderOfName<N> {
    type Target = HeaderValue;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<N> std::ops::DerefMut for RawHeaderOfName<N> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<N> RawHeaderOfName<N> {
    /// 得到内部的值。
    #[inline]
    pub fn into_inner(this: Self) -> HeaderValue {
        this.0
    }
}

impl<N> Clone for RawHeaderOfName<N> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), Default::default())
    }
}

impl<N> std::fmt::Debug for RawHeaderOfName<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("RawHeaderOfName").field(&self.0).finish()
    }
}

impl<N> FromRequest for RawHeaderOfName<N>
where
    N: Name,
{
    type Error = HeaderOfNameExtractError;

    async fn from_request(req: &mut Request) -> Result<Self, Self::Error> {
        find_header_by_name(req, N::name())
            .map(|value| RawHeaderOfName(value.to_owned(), Default::default()))
    }
}

/// 根据标头名获取标头值的提取器。
///
/// `T`需要实现[`FromStr`]。
///
/// # 例子
///
/// ```
/// use boluo::extract::HeaderOfName;
///
/// #[boluo::route("/", method = "GET")]
/// async fn handler(HeaderOfName(host, _): HeaderOfName<name::host, String>) {
///     // ...
/// }
///
/// mod name {
///     #![allow(non_camel_case_types)]
///     boluo::name! {
///         pub host = "host";
///     }
/// }
/// ```
pub struct HeaderOfName<N, T>(pub T, pub PhantomData<fn(N) -> N>);

impl<N, T> std::ops::Deref for HeaderOfName<N, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<N, T> std::ops::DerefMut for HeaderOfName<N, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<N, T> HeaderOfName<N, T> {
    /// 得到内部的值。
    #[inline]
    pub fn into_inner(this: Self) -> T {
        this.0
    }
}

impl<N, T: Clone> Clone for HeaderOfName<N, T> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), Default::default())
    }
}

impl<N, T: Copy> Copy for HeaderOfName<N, T> {}

impl<N, T: std::fmt::Debug> std::fmt::Debug for HeaderOfName<N, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("HeaderOfName").field(&self.0).finish()
    }
}

impl<N, T> FromRequest for HeaderOfName<N, T>
where
    N: Name,
    T: FromStr,
    T::Err: Into<BoxError>,
{
    type Error = HeaderOfNameExtractError;

    async fn from_request(req: &mut Request) -> Result<Self, Self::Error> {
        let name = N::name();

        let value = find_header_by_name(req, name)?;
        let value = percent_encoding::percent_decode(value.as_bytes())
            .decode_utf8()
            .map_err(|e| HeaderOfNameExtractError::ParseError {
                name,
                source: e.into(),
            })?;

        value
            .parse::<T>()
            .map(|value| HeaderOfName(value, Default::default()))
            .map_err(|e| HeaderOfNameExtractError::ParseError {
                name,
                source: e.into(),
            })
    }
}

/// 根据标头名获取原始标头值的提取器，不对标头值进行解析。
///
/// 若标头不存在，则得到[`None`]。
///
/// # 例子
///
/// ```
/// use boluo::extract::OptionalRawHeaderOfName;
///
/// #[boluo::route("/", method = "GET")]
/// async fn handler(OptionalRawHeaderOfName(host, _): OptionalRawHeaderOfName<name::host>) {
///     if let Some(host) = host {
///         // ...
///     }
/// }
///
/// mod name {
///     #![allow(non_camel_case_types)]
///     boluo::name! {
///         pub host = "host";
///     }
/// }
/// ```
pub struct OptionalRawHeaderOfName<N>(pub Option<HeaderValue>, pub PhantomData<fn(N) -> N>);

impl<N> std::ops::Deref for OptionalRawHeaderOfName<N> {
    type Target = Option<HeaderValue>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<N> std::ops::DerefMut for OptionalRawHeaderOfName<N> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<N> OptionalRawHeaderOfName<N> {
    /// 得到内部的值。
    #[inline]
    pub fn into_inner(this: Self) -> Option<HeaderValue> {
        this.0
    }
}

impl<N> Clone for OptionalRawHeaderOfName<N> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), Default::default())
    }
}

impl<N> std::fmt::Debug for OptionalRawHeaderOfName<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("OptionalRawHeaderOfName")
            .field(&self.0)
            .finish()
    }
}

impl<N> FromRequest for OptionalRawHeaderOfName<N>
where
    N: Name,
{
    type Error = HeaderOfNameExtractError;

    async fn from_request(req: &mut Request) -> Result<Self, Self::Error> {
        match RawHeaderOfName::<N>::from_request(req).await {
            Ok(RawHeaderOfName(value, _)) => {
                Ok(OptionalRawHeaderOfName(Some(value), Default::default()))
            }
            Err(HeaderOfNameExtractError::MissingHeader { .. }) => {
                Ok(OptionalRawHeaderOfName(None, Default::default()))
            }
            Err(e) => Err(e),
        }
    }
}

/// 根据标头名获取标头值的提取器。
///
/// `T`需要实现[`FromStr`]。若标头不存在，则得到[`None`]。
///
/// # 例子
///
/// ```
/// use boluo::extract::OptionalHeaderOfName;
///
/// #[boluo::route("/", method = "GET")]
/// async fn handler(OptionalHeaderOfName(host, _): OptionalHeaderOfName<name::host, String>) {
///     if let Some(host) = host {
///         // ...
///     }
/// }
///
/// mod name {
///     #![allow(non_camel_case_types)]
///     boluo::name! {
///         pub host = "host";
///     }
/// }
/// ```
pub struct OptionalHeaderOfName<N, T>(pub Option<T>, pub PhantomData<fn(N) -> N>);

impl<N, T> std::ops::Deref for OptionalHeaderOfName<N, T> {
    type Target = Option<T>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<N, T> std::ops::DerefMut for OptionalHeaderOfName<N, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<N, T> OptionalHeaderOfName<N, T> {
    /// 得到内部的值。
    #[inline]
    pub fn into_inner(this: Self) -> Option<T> {
        this.0
    }
}

impl<N, T: Clone> Clone for OptionalHeaderOfName<N, T> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), Default::default())
    }
}

impl<N, T: Copy> Copy for OptionalHeaderOfName<N, T> {}

impl<N, T: std::fmt::Debug> std::fmt::Debug for OptionalHeaderOfName<N, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("OptionalHeaderOfName")
            .field(&self.0)
            .finish()
    }
}

impl<N, T> FromRequest for OptionalHeaderOfName<N, T>
where
    N: Name,
    T: FromStr,
    T::Err: Into<BoxError>,
{
    type Error = HeaderOfNameExtractError;

    async fn from_request(req: &mut Request) -> Result<Self, Self::Error> {
        match HeaderOfName::<N, T>::from_request(req).await {
            Ok(HeaderOfName(value, _)) => Ok(OptionalHeaderOfName(Some(value), Default::default())),
            Err(HeaderOfNameExtractError::MissingHeader { .. }) => {
                Ok(OptionalHeaderOfName(None, Default::default()))
            }
            Err(e) => Err(e),
        }
    }
}

fn find_header_by_name<'a>(
    req: &'a mut Request,
    name: &'static str,
) -> Result<&'a HeaderValue, HeaderOfNameExtractError> {
    let Ok(header_name) = HeaderName::from_str(name) else {
        return Err(HeaderOfNameExtractError::InvalidHeaderName { name });
    };
    let Some(header_value) = req.headers().get(header_name) else {
        return Err(HeaderOfNameExtractError::MissingHeader { name });
    };
    Ok(header_value)
}

/// 标头提取错误。
#[derive(Debug)]
pub enum HeaderOfNameExtractError {
    /// 标头不存在。
    MissingHeader {
        /// 标头名。
        name: &'static str,
    },
    /// 无效的标头名。
    InvalidHeaderName {
        /// 标头名。
        name: &'static str,
    },
    /// 解析错误。
    ParseError {
        /// 标头名。
        name: &'static str,
        /// 错误源。
        source: BoxError,
    },
}

impl std::fmt::Display for HeaderOfNameExtractError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HeaderOfNameExtractError::MissingHeader { name } => {
                write!(f, "missing request header `{name}`")
            }
            HeaderOfNameExtractError::InvalidHeaderName { name } => {
                write!(f, "invalid request header name `{name}`")
            }
            HeaderOfNameExtractError::ParseError { name, source } => {
                write!(f, "failed to parse request header `{name}` ({source})")
            }
        }
    }
}

impl std::error::Error for HeaderOfNameExtractError {}
