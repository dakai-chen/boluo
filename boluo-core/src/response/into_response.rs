use std::borrow::Cow;
use std::convert::Infallible;

use http::{header, Extensions, HeaderMap, HeaderName, HeaderValue, StatusCode, Version};

use crate::body::{Body, Bytes, HttpBody};
use crate::response::{Response, ResponseParts};
use crate::BoxError;

pub trait IntoResponse {
    type Error: Into<BoxError>;

    fn into_response(self) -> Result<Response, Self::Error>;
}

pub trait IntoResponseParts {
    type Error: Into<BoxError>;

    fn into_response_parts(self, parts: ResponseParts) -> Result<ResponseParts, Self::Error>;
}

impl IntoResponse for Infallible {
    type Error = Infallible;

    fn into_response(self) -> Result<Response, Self::Error> {
        match self {}
    }
}

impl IntoResponse for () {
    type Error = Infallible;

    fn into_response(self) -> Result<Response, Self::Error> {
        Ok(Response::new(Body::empty()))
    }
}

impl IntoResponse for &'static str {
    type Error = Infallible;

    fn into_response(self) -> Result<Response, Self::Error> {
        Cow::Borrowed(self).into_response()
    }
}

impl IntoResponse for String {
    type Error = Infallible;

    fn into_response(self) -> Result<Response, Self::Error> {
        Cow::<'static, str>::Owned(self).into_response()
    }
}

impl IntoResponse for Cow<'static, str> {
    type Error = Infallible;

    fn into_response(self) -> Result<Response, Self::Error> {
        let mut res = Response::new(Body::from(self));
        res.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::TEXT_PLAIN_UTF_8.as_ref()),
        );
        Ok(res)
    }
}

impl IntoResponse for &'static [u8] {
    type Error = Infallible;

    fn into_response(self) -> Result<Response, Self::Error> {
        Cow::Borrowed(self).into_response()
    }
}

impl IntoResponse for Vec<u8> {
    type Error = Infallible;

    fn into_response(self) -> Result<Response, Self::Error> {
        Cow::<'static, [u8]>::Owned(self).into_response()
    }
}

impl IntoResponse for Cow<'static, [u8]> {
    type Error = Infallible;

    fn into_response(self) -> Result<Response, Self::Error> {
        let mut res = Response::new(Body::from(self));
        res.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::APPLICATION_OCTET_STREAM.as_ref()),
        );
        Ok(res)
    }
}

impl IntoResponse for Bytes {
    type Error = Infallible;

    fn into_response(self) -> Result<Response, Self::Error> {
        let mut res = Response::new(Body::from(self));
        res.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::APPLICATION_OCTET_STREAM.as_ref()),
        );
        Ok(res)
    }
}

impl IntoResponse for Body {
    type Error = Infallible;

    fn into_response(self) -> Result<Response, Self::Error> {
        Ok(Response::new(self))
    }
}

impl<B> IntoResponse for Response<B>
where
    B: HttpBody<Data = Bytes> + Send + 'static,
    B::Error: Into<BoxError>,
{
    type Error = Infallible;

    fn into_response(self) -> Result<Response, Self::Error> {
        Ok(self.map(Body::new))
    }
}

impl<R, E> IntoResponse for Result<R, E>
where
    R: IntoResponse,
    E: Into<BoxError>,
{
    type Error = BoxError;

    fn into_response(self) -> Result<Response, Self::Error> {
        self.map_or_else(|e| Err(e.into()), |r| r.into_response().map_err(Into::into))
    }
}

impl<P> IntoResponse for P
where
    P: IntoResponseParts,
{
    type Error = P::Error;

    fn into_response(self) -> Result<Response, Self::Error> {
        let parts = Response::new(()).into_parts();
        self.into_response_parts(parts)
            .map(|parts| Response::from_parts(parts, Body::empty()))
    }
}

macro_rules! into_response_tuples {
    ($($ty:ident),* @ $($tyr:ident),* $(,)?) => {
        #[allow(non_snake_case)]
        impl<R, $($ty,)*> IntoResponse for ($($ty),*, R)
        where
            $($ty: IntoResponseParts,)*
            R: IntoResponse,
        {
            type Error = BoxError;

            fn into_response(self) -> Result<Response, Self::Error> {
                let ($($ty),*, res) = self;

                let res = res.into_response().map_err(Into::into)?;
                let (parts, body) = res.into_inner();
                $(
                    let parts = $tyr.into_response_parts(parts).map_err(Into::into)?;
                )*
                Ok(Response::from_parts(parts, body))
            }
        }
    }
}

into_response_tuples!(
    T1 @
    T1
);
into_response_tuples!(
    T1, T2 @
    T2, T1
);
into_response_tuples!(
    T1, T2, T3 @
    T3, T2, T1
);
into_response_tuples!(
    T1, T2, T3, T4 @
    T4, T3, T2, T1
);
into_response_tuples!(
    T1, T2, T3, T4, T5 @
    T5, T4, T3, T2, T1
);
into_response_tuples!(
    T1, T2, T3, T4, T5, T6 @
    T6, T5, T4, T3, T2, T1
);
into_response_tuples!(
    T1, T2, T3, T4, T5, T6, T7 @
    T7, T6, T5, T4, T3, T2, T1
);
into_response_tuples!(
    T1, T2, T3, T4, T5, T6, T7, T8 @
    T8, T7, T6, T5, T4, T3, T2, T1
);
into_response_tuples!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9 @
    T9, T8, T7, T6, T5, T4, T3, T2, T1
);
into_response_tuples!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10 @
    T10, T9, T8, T7, T6, T5, T4, T3, T2, T1
);
into_response_tuples!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11 @
    T11, T10, T9, T8, T7, T6, T5, T4, T3, T2, T1
);
into_response_tuples!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12 @
    T12, T11, T10, T9, T8, T7, T6, T5, T4, T3, T2, T1
);
into_response_tuples!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13 @
    T13, T12, T11, T10, T9, T8, T7, T6, T5, T4, T3, T2, T1
);
into_response_tuples!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14 @
    T14, T13, T12, T11, T10, T9, T8, T7, T6, T5, T4, T3, T2, T1
);
into_response_tuples!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15 @
    T15, T14, T13, T12, T11, T10, T9, T8, T7, T6, T5, T4, T3, T2, T1
);
into_response_tuples!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16 @
    T16, T15, T14, T13, T12, T11, T10, T9, T8, T7, T6, T5, T4, T3, T2, T1
);

impl<T> IntoResponseParts for Option<T>
where
    T: IntoResponseParts,
{
    type Error = T::Error;

    fn into_response_parts(self, parts: ResponseParts) -> Result<ResponseParts, Self::Error> {
        match self {
            Some(this) => this.into_response_parts(parts),
            None => Ok(parts),
        }
    }
}

impl IntoResponseParts for StatusCode {
    type Error = Infallible;

    fn into_response_parts(self, mut parts: ResponseParts) -> Result<ResponseParts, Self::Error> {
        parts.status = self;
        Ok(parts)
    }
}

impl IntoResponseParts for Version {
    type Error = Infallible;

    fn into_response_parts(self, mut parts: ResponseParts) -> Result<ResponseParts, Self::Error> {
        parts.version = self;
        Ok(parts)
    }
}

impl IntoResponseParts for HeaderMap {
    type Error = Infallible;

    fn into_response_parts(self, mut parts: ResponseParts) -> Result<ResponseParts, Self::Error> {
        parts.headers.extend(self);
        Ok(parts)
    }
}

impl IntoResponseParts for Extensions {
    type Error = Infallible;

    fn into_response_parts(self, mut parts: ResponseParts) -> Result<ResponseParts, Self::Error> {
        parts.extensions.extend(self);
        Ok(parts)
    }
}

impl IntoResponseParts for ResponseParts {
    type Error = Infallible;

    fn into_response_parts(self, mut parts: ResponseParts) -> Result<ResponseParts, Self::Error> {
        parts = self.extensions.into_response_parts(parts)?;
        parts = self.headers.into_response_parts(parts)?;
        parts = self.version.into_response_parts(parts)?;
        parts = self.status.into_response_parts(parts)?;
        Ok(parts)
    }
}

impl<K, V, const N: usize> IntoResponseParts for [(K, V); N]
where
    K: TryInto<HeaderName>,
    K::Error: std::fmt::Display,
    V: TryInto<HeaderValue>,
    V::Error: std::fmt::Display,
{
    type Error = IntoHeaderError;

    fn into_response_parts(self, mut parts: ResponseParts) -> Result<ResponseParts, Self::Error> {
        for (k, v) in self {
            let k = k
                .try_into()
                .map_err(|e| IntoHeaderError::InvalidName(e.to_string()))?;
            let v = v
                .try_into()
                .map_err(|e| IntoHeaderError::InvalidValue(e.to_string()))?;
            parts.headers.insert(k, v);
        }
        Ok(parts)
    }
}

impl<K, V> IntoResponseParts for Vec<(K, V)>
where
    K: TryInto<HeaderName>,
    K::Error: std::fmt::Display,
    V: TryInto<HeaderValue>,
    V::Error: std::fmt::Display,
{
    type Error = IntoHeaderError;

    fn into_response_parts(self, mut parts: ResponseParts) -> Result<ResponseParts, Self::Error> {
        for (k, v) in self {
            let k = k
                .try_into()
                .map_err(|e| IntoHeaderError::InvalidName(e.to_string()))?;
            let v = v
                .try_into()
                .map_err(|e| IntoHeaderError::InvalidValue(e.to_string()))?;
            parts.headers.insert(k, v);
        }
        Ok(parts)
    }
}

#[derive(Debug)]
pub enum IntoHeaderError {
    InvalidName(String),
    InvalidValue(String),
}

impl std::fmt::Display for IntoHeaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IntoHeaderError::InvalidName(e) => {
                write!(f, "invalid header name ({e})")
            }
            IntoHeaderError::InvalidValue(e) => {
                write!(f, "invalid header value ({e})")
            }
        }
    }
}

impl std::error::Error for IntoHeaderError {}
