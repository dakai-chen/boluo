//! 可用于处理请求并返回响应的异步函数。

use std::marker::PhantomData;

use crate::BoxError;
use crate::extract::FromRequest;
use crate::request::Request;
use crate::response::{IntoResponse, Response};
use crate::service::Service;

/// 将给定的处理程序转换为 [`Service`]。
///
/// # 例子
///
/// ```
/// use boluo_core::handler::handler_fn;
///
/// async fn hello() -> &'static str {
///     "Hello, World!"
/// }
///
/// let service = handler_fn(hello);
/// ```
pub fn handler_fn<F, T>(f: F) -> HandlerFn<F, T>
where
    HandlerFn<F, T>: Service<Request>,
{
    HandlerFn {
        f,
        _marker: Default::default(),
    }
}

/// 将给定的处理程序转换为 [`Service`]。
pub struct HandlerFn<F, T> {
    f: F,
    _marker: PhantomData<fn(T) -> T>,
}

impl<F: Clone, T> Clone for HandlerFn<F, T> {
    fn clone(&self) -> Self {
        Self {
            f: self.f.clone(),
            _marker: Default::default(),
        }
    }
}

impl<F: Copy, T> Copy for HandlerFn<F, T> {}

impl<F, T> std::fmt::Debug for HandlerFn<F, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HandlerFn")
            .field("f", &std::any::type_name::<F>())
            .finish()
    }
}

impl<F, Fut> Service<Request> for HandlerFn<F, ()>
where
    F: Fn() -> Fut + Send + Sync,
    Fut: Future + Send,
    Fut::Output: IntoResponse,
{
    type Response = Response;
    type Error = BoxError;

    async fn call(&self, _: Request) -> Result<Self::Response, Self::Error> {
        (self.f)().await.into_response().map_err(Into::into)
    }
}

impl<F, Fut> Service<Request> for HandlerFn<F, Request>
where
    F: Fn(Request) -> Fut + Send + Sync,
    Fut: Future + Send,
    Fut::Output: IntoResponse,
{
    type Response = Response;
    type Error = BoxError;

    async fn call(&self, request: Request) -> Result<Self::Response, Self::Error> {
        (self.f)(request).await.into_response().map_err(Into::into)
    }
}

macro_rules! handler_tuples {
    ($($ty:ident),*) => {
        #[allow(non_snake_case)]
        impl<F, Fut, $($ty,)*> Service<Request> for HandlerFn<F, ($($ty,)*)>
        where
            F: Fn($($ty,)*) -> Fut + Send + Sync,
            Fut: Future + Send,
            Fut::Output: IntoResponse,
            $($ty: FromRequest + Send,)*
            $(<$ty as FromRequest>::Error: Into<BoxError>,)*
        {
            type Response = Response;
            type Error = BoxError;

            async fn call(&self, mut request: Request) -> Result<Self::Response, Self::Error> {
                $(
                    let $ty = $ty::from_request(&mut request).await.map_err(Into::into)?;
                )*
                (self.f)($($ty,)*).await.into_response().map_err(Into::into)
            }
        }

        #[allow(non_snake_case)]
        impl<F, Fut, $($ty,)*> Service<Request> for HandlerFn<F, ($($ty,)* Request)>
        where
            F: Fn($($ty,)* Request) -> Fut + Send + Sync,
            Fut: Future + Send,
            Fut::Output: IntoResponse,
            $($ty: FromRequest + Send,)*
            $(<$ty as FromRequest>::Error: Into<BoxError>,)*
        {
            type Response = Response;
            type Error = BoxError;

            async fn call(&self, mut request: Request) -> Result<Self::Response, Self::Error> {
                $(
                    let $ty = $ty::from_request(&mut request).await.map_err(Into::into)?;
                )*
                (self.f)($($ty,)* request).await.into_response().map_err(Into::into)
            }
        }
    };
}

handler_tuples!(T1);
handler_tuples!(T1, T2);
handler_tuples!(T1, T2, T3);
handler_tuples!(T1, T2, T3, T4);
handler_tuples!(T1, T2, T3, T4, T5);
handler_tuples!(T1, T2, T3, T4, T5, T6);
handler_tuples!(T1, T2, T3, T4, T5, T6, T7);
handler_tuples!(T1, T2, T3, T4, T5, T6, T7, T8);
handler_tuples!(T1, T2, T3, T4, T5, T6, T7, T8, T9);
handler_tuples!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
handler_tuples!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
handler_tuples!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);
handler_tuples!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13);
handler_tuples!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14);
handler_tuples!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15
);
handler_tuples!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16
);
