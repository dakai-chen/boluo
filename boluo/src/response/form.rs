use boluo_core::body::Body;
use boluo_core::http::{header, HeaderValue};
use boluo_core::response::{IntoResponse, Response};
use serde::Serialize;

pub use crate::data::Form;

impl<T> IntoResponse for Form<T>
where
    T: Serialize,
{
    type Error = ResponseFormError;

    fn into_response(self) -> Result<Response, Self::Error> {
        let data = serde_urlencoded::to_string(&self.0)?;
        let mut res = Response::new(Body::from(data));
        res.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::APPLICATION_WWW_FORM_URLENCODED.as_ref()),
        );
        Ok(res)
    }
}

#[derive(Debug)]
pub struct ResponseFormError(pub serde_urlencoded::ser::Error);

impl std::fmt::Display for ResponseFormError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "error serializing to form ({})", self.0)
    }
}

impl std::error::Error for ResponseFormError {}

impl From<serde_urlencoded::ser::Error> for ResponseFormError {
    fn from(error: serde_urlencoded::ser::Error) -> Self {
        Self(error)
    }
}
