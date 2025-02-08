use boluo_core::BoxError;
use boluo_core::body::Bytes;
use boluo_core::extract::FromRequest;
use boluo_core::http::{HeaderMap, header};
use boluo_core::request::Request;
use serde::de::DeserializeOwned;

pub use crate::data::Json;

impl<T> FromRequest for Json<T>
where
    T: DeserializeOwned,
{
    type Error = JsonExtractError;

    async fn from_request(req: &mut Request) -> Result<Self, Self::Error> {
        if !is_json_content_type(req.headers()) {
            return Err(JsonExtractError::UnsupportedContentType);
        }

        let bytes = Bytes::from_request(req)
            .await
            .map_err(JsonExtractError::FailedToBufferBody)?;

        serde_json::from_slice::<T>(&bytes)
            .map(|value| Json(value))
            .map_err(JsonExtractError::FailedToDeserialize)
    }
}

fn is_json_content_type(headers: &HeaderMap) -> bool {
    let Some(content_type) = headers.get(header::CONTENT_TYPE) else {
        return false;
    };
    let Ok(content_type) = content_type.to_str() else {
        return false;
    };

    let Ok(mime) = content_type.parse::<mime::Mime>() else {
        return false;
    };

    let is_json_content_type = mime.type_() == "application"
        && (mime.subtype() == "json" || mime.suffix().filter(|name| *name == "json").is_some());

    is_json_content_type
}

/// JSON提取错误。
#[derive(Debug)]
pub enum JsonExtractError {
    /// 不支持的内容类型。
    UnsupportedContentType,
    /// 缓冲主体失败。
    FailedToBufferBody(BoxError),
    /// 反序列化失败。
    FailedToDeserialize(serde_json::Error),
}

impl std::fmt::Display for JsonExtractError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JsonExtractError::UnsupportedContentType => f.write_str("unsupported content type"),
            JsonExtractError::FailedToBufferBody(e) => write!(f, "failed to buffer body ({e})"),
            JsonExtractError::FailedToDeserialize(e) => {
                write!(f, "failed to deserialize json ({e})")
            }
        }
    }
}

impl std::error::Error for JsonExtractError {}
