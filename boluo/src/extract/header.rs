use std::ops::{Deref, DerefMut};

use boluo_core::extract::FromRequest;
use boluo_core::http::HeaderName;
use boluo_core::request::Request;
use headers::{Header, HeaderMapExt};

/// 获取请求标头值的提取器。
///
/// `T`需要实现[`Header`]。
///
/// # 例子
///
/// ```
/// use boluo::extract::TypedHeader;
/// use boluo::headers::Host;
///
/// #[boluo::route("/", method = "GET")]
/// async fn handler(TypedHeader(host): TypedHeader<Host>) {
///     // ...
/// }
/// ```
#[derive(Debug, Clone, Copy)]
pub struct TypedHeader<T>(pub T);

impl<T> Deref for TypedHeader<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for TypedHeader<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> TypedHeader<T> {
    /// 得到内部的值。
    #[inline]
    pub fn into_inner(this: Self) -> T {
        this.0
    }
}

impl<T> FromRequest for TypedHeader<T>
where
    T: Header,
{
    type Error = TypedHeaderExtractError;

    async fn from_request(req: &mut Request) -> Result<Self, Self::Error> {
        let OptionalTypedHeader(value) = OptionalTypedHeader::from_request(req).await?;
        Ok(TypedHeader(value.ok_or_else(|| {
            TypedHeaderExtractError::MissingHeader { name: T::name() }
        })?))
    }
}

/// 获取请求标头值的提取器。
///
/// `T`需要实现[`Header`]。若标头不存在，则得到[`None`]。
///
/// # 例子
///
/// ```
/// use boluo::extract::OptionalTypedHeader;
/// use boluo::headers::Host;
///
/// #[boluo::route("/", method = "GET")]
/// async fn handler(OptionalTypedHeader(host): OptionalTypedHeader<Host>) {
///     if let Some(host) = host {
///         // ...
///     }
/// }
/// ```
#[derive(Debug, Clone, Copy)]
pub struct OptionalTypedHeader<T>(pub Option<T>);

impl<T> Deref for OptionalTypedHeader<T> {
    type Target = Option<T>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for OptionalTypedHeader<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> OptionalTypedHeader<T> {
    /// 得到内部的值。
    #[inline]
    pub fn into_inner(this: Self) -> Option<T> {
        this.0
    }
}

impl<T> FromRequest for OptionalTypedHeader<T>
where
    T: Header,
{
    type Error = TypedHeaderExtractError;

    async fn from_request(req: &mut Request) -> Result<Self, Self::Error> {
        req.headers()
            .typed_try_get()
            .map(OptionalTypedHeader)
            .map_err(|source| TypedHeaderExtractError::ParseError {
                name: T::name(),
                source,
            })
    }
}

/// 获取请求标头值错误。
#[derive(Debug)]
pub enum TypedHeaderExtractError {
    /// 标头不存在。
    MissingHeader {
        /// 标头名。
        name: &'static HeaderName,
    },
    /// 解析错误。
    ParseError {
        /// 标头名。
        name: &'static HeaderName,
        /// 错误源。
        source: headers::Error,
    },
}

impl std::fmt::Display for TypedHeaderExtractError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypedHeaderExtractError::MissingHeader { name } => {
                write!(f, "missing request header `{name}`")
            }
            TypedHeaderExtractError::ParseError { name, source } => {
                write!(f, "failed to parse request header `{name}` ({source})")
            }
        }
    }
}

impl std::error::Error for TypedHeaderExtractError {}
