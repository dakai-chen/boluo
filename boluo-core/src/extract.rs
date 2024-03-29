//! 从请求中提取数据的类型和特征。

use std::convert::Infallible;
use std::future::Future;

use http::{Extensions, HeaderMap, Method, Uri, Version};
use http_body_util::BodyExt;

use crate::body::{Body, Bytes};
use crate::request::{Request, RequestParts};
use crate::BoxError;

/// 可以根据[`Request`]创建的类型，用于实现提取器。
///
/// # 例子
///
/// ```
/// use std::convert::Infallible;
///
/// use boluo_core::extract::FromRequest;
/// use boluo_core::http::{header, HeaderValue};
/// use boluo_core::request::Request;
///
/// // 从请求头中提取HOST的提取器。
/// struct Host(Option<HeaderValue>);
///
/// // 为提取器实现`FromRequest`特征。
/// impl FromRequest for Host {
///     type Error = Infallible;
///
///     async fn from_request(req: &mut Request) -> Result<Self, Self::Error> {
///         let value = req.headers().get(header::HOST).map(|v| v.to_owned());
///         Ok(Host(value))
///     }
/// }
///
/// // 在处理程序中使用提取器从请求中提取数据。
/// async fn using_extractor(Host(host): Host) {
///     println!("{host:?}")
/// }
/// ```
pub trait FromRequest: Sized {
    /// 提取器的错误类型。
    type Error;

    /// 根据[`Request`]创建提取器实例。
    fn from_request(req: &mut Request) -> impl Future<Output = Result<Self, Self::Error>> + Send;
}

impl<T> FromRequest for Option<T>
where
    T: FromRequest,
{
    type Error = Infallible;

    async fn from_request(req: &mut Request) -> Result<Self, Self::Error> {
        Ok(T::from_request(req).await.ok())
    }
}

impl<T> FromRequest for Result<T, T::Error>
where
    T: FromRequest,
{
    type Error = Infallible;

    async fn from_request(req: &mut Request) -> Result<Self, Self::Error> {
        Ok(T::from_request(req).await)
    }
}

macro_rules! from_request_tuples {
    ($($ty:ident),*) => {
        #[allow(non_snake_case)]
        impl<$($ty,)*> FromRequest for ($($ty,)*)
        where
            $($ty: FromRequest + Send,)*
            $(<$ty as FromRequest>::Error: Into<BoxError>,)*
        {
            type Error = BoxError;

            async fn from_request(req: &mut Request) -> Result<Self, Self::Error> {
                $(
                    let $ty = $ty::from_request(req).await.map_err(Into::into)?;
                )*
                Ok(($($ty,)*))
            }
        }
    };
}

from_request_tuples!(T1);
from_request_tuples!(T1, T2);
from_request_tuples!(T1, T2, T3);
from_request_tuples!(T1, T2, T3, T4);
from_request_tuples!(T1, T2, T3, T4, T5);
from_request_tuples!(T1, T2, T3, T4, T5, T6);
from_request_tuples!(T1, T2, T3, T4, T5, T6, T7);
from_request_tuples!(T1, T2, T3, T4, T5, T6, T7, T8);
from_request_tuples!(T1, T2, T3, T4, T5, T6, T7, T8, T9);
from_request_tuples!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
from_request_tuples!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
from_request_tuples!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);
from_request_tuples!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13);
from_request_tuples!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14);
from_request_tuples!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15);
from_request_tuples!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16);

impl FromRequest for Body {
    type Error = Infallible;

    async fn from_request(req: &mut Request) -> Result<Self, Self::Error> {
        Ok(std::mem::take(req.body_mut()))
    }
}

impl FromRequest for Bytes {
    type Error = BoxError;

    async fn from_request(req: &mut Request) -> Result<Self, Self::Error> {
        req.body_mut().collect().await.map(|col| col.to_bytes())
    }
}

impl FromRequest for String {
    type Error = BoxError;

    async fn from_request(req: &mut Request) -> Result<Self, Self::Error> {
        let bytes = Bytes::from_request(req).await?;
        Ok(std::str::from_utf8(&bytes)?.to_owned())
    }
}

impl FromRequest for Method {
    type Error = Infallible;

    async fn from_request(req: &mut Request) -> Result<Self, Self::Error> {
        Ok(req.method().clone())
    }
}

impl FromRequest for Uri {
    type Error = Infallible;

    async fn from_request(req: &mut Request) -> Result<Self, Self::Error> {
        Ok(req.uri().clone())
    }
}

impl FromRequest for Version {
    type Error = Infallible;

    async fn from_request(req: &mut Request) -> Result<Self, Self::Error> {
        Ok(req.version())
    }
}

impl FromRequest for HeaderMap {
    type Error = Infallible;

    async fn from_request(req: &mut Request) -> Result<Self, Self::Error> {
        Ok(req.headers().clone())
    }
}

impl FromRequest for Extensions {
    type Error = Infallible;

    async fn from_request(req: &mut Request) -> Result<Self, Self::Error> {
        Ok(req.extensions().clone())
    }
}

impl FromRequest for RequestParts {
    type Error = Infallible;

    async fn from_request(req: &mut Request) -> Result<Self, Self::Error> {
        Ok(req.parts().clone())
    }
}

/// 用来表示一个常量字符串的特征。
///
/// 常量泛型目前还不支持使用常量字符串，所以无法直接使用常量字符串作为提取器的泛型参数。
/// 通过该特征将常量字符串表示为类型，从而可以在提取器中使用常量字符串作为提取的索引。
///
/// 该特征将在常量泛型支持使用常量字符串时移除，并对涉及的提取器进行修改。
///
/// # 例子
///
/// ```
/// use boluo_core::extract::Name;
///
/// struct ContentType;
///
/// impl Name for ContentType {
///     fn name() -> &'static str {
///         "content-type"
///     }
/// }
///
/// assert_eq!(ContentType::name(), "content-type");
/// ```
pub trait Name {
    /// 获取常量字符串。
    fn name() -> &'static str;
}
