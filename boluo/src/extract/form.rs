use boluo_core::BoxError;
use boluo_core::body::Bytes;
use boluo_core::extract::FromRequest;
use boluo_core::http::{HeaderMap, Method, header};
use boluo_core::request::Request;
use serde::de::DeserializeOwned;

pub use crate::data::Form;

use super::{Query, QueryError};

impl<T> FromRequest for Form<T>
where
    T: DeserializeOwned,
{
    type Error = FormError;

    async fn from_request(request: &mut Request) -> Result<Self, Self::Error> {
        if request.method() == Method::GET || request.method() == Method::HEAD {
            Query::from_request(request)
                .await
                .map(|Query(value)| Form(value))
                .map_err(FormError::from_extract_query_error)
        } else {
            if !has_content_type(request.headers(), &mime::APPLICATION_WWW_FORM_URLENCODED) {
                return Err(FormError::UnsupportedContentType);
            }

            let bytes = Bytes::from_request(request)
                .await
                .map_err(FormError::FailedToBufferBody)?;

            serde_urlencoded::from_bytes::<T>(&bytes)
                .map(|value| Form(value))
                .map_err(FormError::FailedToDeserialize)
        }
    }
}

fn has_content_type(headers: &HeaderMap, expected_content_type: &mime::Mime) -> bool {
    let Some(content_type) = headers.get(header::CONTENT_TYPE) else {
        return false;
    };
    let Ok(content_type) = content_type.to_str() else {
        return false;
    };

    content_type.starts_with(expected_content_type.as_ref())
}

/// 表单提取错误。
#[derive(Debug)]
pub enum FormError {
    /// 不支持的内容类型。
    UnsupportedContentType,
    /// 缓冲主体失败。
    FailedToBufferBody(BoxError),
    /// 反序列化失败。
    FailedToDeserialize(serde_urlencoded::de::Error),
}

impl FormError {
    fn from_extract_query_error(error: QueryError) -> Self {
        match error {
            QueryError::FailedToDeserialize(e) => FormError::FailedToDeserialize(e),
        }
    }
}

impl std::fmt::Display for FormError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FormError::UnsupportedContentType => f.write_str("unsupported content type"),
            FormError::FailedToBufferBody(e) => write!(f, "failed to buffer body ({e})"),
            FormError::FailedToDeserialize(e) => {
                write!(f, "failed to deserialize form ({e})")
            }
        }
    }
}

impl std::error::Error for FormError {}
