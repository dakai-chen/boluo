use boluo_core::body::Bytes;
use boluo_core::extract::FromRequest;
use boluo_core::http::{header, HeaderMap, Method};
use boluo_core::request::Request;
use boluo_core::BoxError;
use serde::de::DeserializeOwned;

pub use crate::data::Form;

use super::{ExtractQueryError, Query};

impl<T> FromRequest for Form<T>
where
    T: DeserializeOwned,
{
    type Error = ExtractFormError;

    async fn from_request(req: &mut Request) -> Result<Self, Self::Error> {
        if req.method() == Method::GET {
            Query::from_request(req)
                .await
                .map(|Query(value)| Form(value))
                .map_err(ExtractFormError::from_extract_query_error)
        } else {
            if !has_content_type(req.headers(), &mime::APPLICATION_WWW_FORM_URLENCODED) {
                return Err(ExtractFormError::UnsupportedContentType);
            }

            let bytes = Bytes::from_request(req)
                .await
                .map_err(|e| ExtractFormError::FailedToReadBody(e.into()))?;

            serde_urlencoded::from_bytes::<T>(&bytes)
                .map(|value| Form(value))
                .map_err(ExtractFormError::FailedToDeserialize)
        }
    }
}

fn has_content_type(headers: &HeaderMap, expected_content_type: &mime::Mime) -> bool {
    let content_type = if let Some(content_type) = headers.get(header::CONTENT_TYPE) {
        content_type
    } else {
        return false;
    };

    let content_type = if let Ok(content_type) = content_type.to_str() {
        content_type
    } else {
        return false;
    };

    content_type.starts_with(expected_content_type.as_ref())
}

#[derive(Debug)]
pub enum ExtractFormError {
    UnsupportedContentType,
    FailedToReadBody(BoxError),
    FailedToDeserialize(serde_urlencoded::de::Error),
}

impl ExtractFormError {
    fn from_extract_query_error(error: ExtractQueryError) -> Self {
        match error {
            ExtractQueryError::FailedToDeserialize(e) => ExtractFormError::FailedToDeserialize(e),
        }
    }
}

impl std::fmt::Display for ExtractFormError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExtractFormError::UnsupportedContentType => f.write_str("unsupported content type"),
            ExtractFormError::FailedToReadBody(e) => write!(f, "failed to read body ({e})"),
            ExtractFormError::FailedToDeserialize(e) => {
                write!(f, "failed to deserialize ({e})")
            }
        }
    }
}

impl std::error::Error for ExtractFormError {}
