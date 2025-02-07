use boluo_core::BoxError;
use boluo_core::body::Bytes;
use boluo_core::extract::FromRequest;
use boluo_core::http::{HeaderMap, Method, header};
use boluo_core::request::Request;
use serde::de::DeserializeOwned;

pub use crate::data::Form;

use super::{Query, QueryExtractError};

impl<T> FromRequest for Form<T>
where
    T: DeserializeOwned,
{
    type Error = FormExtractError;

    async fn from_request(req: &mut Request) -> Result<Self, Self::Error> {
        if req.method() == Method::GET || req.method() == Method::HEAD {
            Query::from_request(req)
                .await
                .map(|Query(value)| Form(value))
                .map_err(FormExtractError::from_extract_query_error)
        } else {
            if !has_content_type(req.headers(), &mime::APPLICATION_WWW_FORM_URLENCODED) {
                return Err(FormExtractError::UnsupportedContentType);
            }

            let bytes = Bytes::from_request(req)
                .await
                .map_err(FormExtractError::FailedToBufferBody)?;

            serde_urlencoded::from_bytes::<T>(&bytes)
                .map(|value| Form(value))
                .map_err(FormExtractError::FailedToDeserialize)
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
pub enum FormExtractError {
    /// 不支持的内容类型。
    UnsupportedContentType,
    /// 缓冲主体失败。
    FailedToBufferBody(BoxError),
    /// 反序列化失败。
    FailedToDeserialize(serde_urlencoded::de::Error),
}

impl FormExtractError {
    fn from_extract_query_error(error: QueryExtractError) -> Self {
        match error {
            QueryExtractError::FailedToDeserialize(e) => FormExtractError::FailedToDeserialize(e),
        }
    }
}

impl std::fmt::Display for FormExtractError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FormExtractError::UnsupportedContentType => f.write_str("unsupported content type"),
            FormExtractError::FailedToBufferBody(e) => write!(f, "failed to buffer body ({e})"),
            FormExtractError::FailedToDeserialize(e) => {
                write!(f, "failed to deserialize form ({e})")
            }
        }
    }
}

impl std::error::Error for FormExtractError {}
