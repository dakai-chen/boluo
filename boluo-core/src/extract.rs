//! 从请求中提取数据的类型和特征。

use std::convert::Infallible;
use std::future::Future;

use http::{Extensions, HeaderMap, Method, Uri, Version};
use http_body_util::BodyExt;

use crate::BoxError;
use crate::body::{Body, Bytes};
use crate::request::{Request, RequestParts};

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

/// 可以根据[`Request`]创建的类型，用于实现提取器。
///
/// 与[`FromRequest`]不同的是，如果提取的数据不存在，则返回`Ok(None)`。
///
/// # 例子
///
/// ```
/// use std::convert::Infallible;
///
/// use boluo_core::extract::OptionalFromRequest;
/// use boluo_core::http::{header, HeaderValue};
/// use boluo_core::request::Request;
///
/// // 从请求头中提取HOST的提取器。
/// struct Host(HeaderValue);
///
/// // 为提取器实现`OptionalFromRequest`特征。
/// impl OptionalFromRequest for Host {
///     type Error = Infallible;
///
///     async fn from_request(req: &mut Request) -> Result<Option<Self>, Self::Error> {
///         Ok(req.headers().get(header::HOST).map(|v| Host(v.to_owned())))
///     }
/// }
///
/// // 在处理程序中使用提取器从请求中提取数据。
/// async fn using_extractor(host: Option<Host>) {
///     if let Some(Host(host)) = host {
///         println!("{host:?}")
///     }
/// }
/// ```
pub trait OptionalFromRequest: Sized {
    /// 提取器的错误类型。
    type Error;

    /// 根据[`Request`]创建提取器实例。
    fn from_request(
        req: &mut Request,
    ) -> impl Future<Output = Result<Option<Self>, Self::Error>> + Send;
}

impl<T> FromRequest for Option<T>
where
    T: OptionalFromRequest,
{
    type Error = T::Error;

    async fn from_request(req: &mut Request) -> Result<Self, Self::Error> {
        T::from_request(req).await
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
from_request_tuples!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15
);
from_request_tuples!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16
);

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

/// 简化[`Result`]提取器的书写。
pub type ExtractResult<T> = std::result::Result<T, <T as FromRequest>::Error>;
