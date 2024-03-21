use boluo_core::body::Body;
use boluo_core::http::{header, HeaderValue};
use boluo_core::response::{IntoResponse, Response};
use serde::Serialize;

pub use crate::data::Json;

impl<T> IntoResponse for Json<T>
where
    T: Serialize,
{
    type Error = JsonResponseError;

    fn into_response(self) -> Result<Response, Self::Error> {
        let data = serde_json::to_vec(&self.0)?;
        let mut res = Response::new(Body::from(data));
        res.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::APPLICATION_JSON.as_ref()),
        );
        Ok(res)
    }
}

#[derive(Debug)]
pub enum JsonResponseError {
    FailedToSerialize(serde_json::Error),
}

impl std::fmt::Display for JsonResponseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JsonResponseError::FailedToSerialize(e) => {
                write!(f, "failed to serialize json ({e})")
            }
        }
    }
}

impl std::error::Error for JsonResponseError {}

impl From<serde_json::Error> for JsonResponseError {
    fn from(error: serde_json::Error) -> Self {
        JsonResponseError::FailedToSerialize(error)
    }
}
