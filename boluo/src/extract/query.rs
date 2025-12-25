use std::convert::Infallible;
use std::ops::{Deref, DerefMut};

use boluo_core::extract::FromRequest;
use boluo_core::request::Request;
use serde::de::DeserializeOwned;

/// 将查询字符串反序列化为某种类型的提取器。
///
/// `T` 需要实现 [`serde::de::DeserializeOwned`]。
///
/// # 例子
///
/// ```
/// use boluo::extract::Query;
///
/// #[derive(serde::Deserialize)]
/// struct Pagination {
///     page: usize,
///     per_page: usize,
/// }
///
/// #[boluo::route("/list_things", method = "GET")]
/// async fn list_things(Query(pagination): Query<Pagination>) {
///     // ...
/// }
/// ```
#[derive(Debug, Clone, Copy)]
pub struct Query<T>(pub T);

impl<T> Deref for Query<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Query<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> Query<T> {
    /// 得到内部的值。
    #[inline]
    pub fn into_inner(this: Self) -> T {
        this.0
    }
}

impl<T> FromRequest for Query<T>
where
    T: DeserializeOwned,
{
    type Error = QueryError;

    async fn from_request(request: &mut Request) -> Result<Self, Self::Error> {
        let query = request.uri().query().unwrap_or_default();
        serde_urlencoded::from_str::<T>(query)
            .map(|value| Query(value))
            .map_err(QueryError::FailedToDeserialize)
    }
}

/// 获取原始查询字符串的提取器，不对查询字符串进行解析。
///
/// # 例子
///
/// ```
/// use boluo::extract::RawQuery;
///
/// #[boluo::route("/", method = "GET")]
/// async fn handler(RawQuery(query): RawQuery) {
///     // ...
/// }
/// ```
#[derive(Debug, Clone)]
pub struct RawQuery(pub String);

impl Deref for RawQuery {
    type Target = String;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RawQuery {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl RawQuery {
    /// 得到内部的值。
    #[inline]
    pub fn into_inner(this: Self) -> String {
        this.0
    }
}

impl FromRequest for RawQuery {
    type Error = Infallible;

    async fn from_request(request: &mut Request) -> Result<Self, Self::Error> {
        Ok(RawQuery(
            request.uri().query().unwrap_or_default().to_owned(),
        ))
    }
}

/// 查询字符串提取错误。
#[derive(Debug)]
pub enum QueryError {
    /// 反序列化错误。
    FailedToDeserialize(serde_urlencoded::de::Error),
}

impl std::fmt::Display for QueryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueryError::FailedToDeserialize(e) => {
                write!(f, "failed to deserialize query string ({e})")
            }
        }
    }
}

impl std::error::Error for QueryError {}
