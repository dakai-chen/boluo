//! 用于解析文件上传中常用的 `multipart/form-data` 格式数据。

use std::pin::Pin;
use std::task::{Context, Poll};

use boluo_core::body::Bytes;
use boluo_core::extract::FromRequest;
use boluo_core::http::HeaderMap;
use boluo_core::http::header::CONTENT_TYPE;
use boluo_core::request::Request;
use futures_util::Stream;

/// 解析 `multipart/form-data` 请求的提取器。
///
/// # 例子
///
/// ```
/// use boluo::multipart::Multipart;
///
/// #[boluo::route("/upload", method = "POST")]
/// async fn upload(mut multipart: Multipart) {
///     while let Ok(Some(field)) = multipart.next_field().await {
///         let name = field.name().unwrap_or_default().to_owned();
///         let data = field.bytes().await.unwrap();
///
///         println!("Length of `{}` is {} bytes", name, data.len());
///     }
/// }
/// ```
#[derive(Debug)]
pub struct Multipart {
    inner: multer::Multipart<'static>,
}

impl Multipart {
    fn parse_boundary(headers: &HeaderMap) -> Option<String> {
        headers
            .get(&CONTENT_TYPE)?
            .to_str()
            .ok()
            .and_then(|content_type| multer::parse_boundary(content_type).ok())
    }
}

impl Multipart {
    /// 生成下一个 [`Field`]。
    pub async fn next_field(&mut self) -> Result<Option<Field>, MultipartError> {
        let field = self
            .inner
            .next_field()
            .await
            .map_err(MultipartError::from_multer)?;

        Ok(field.map(|f| Field { inner: f }))
    }
}

impl Stream for Multipart {
    type Item = Result<Field, MultipartError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        std::pin::pin!(self.next_field())
            .poll(cx)
            .map(|f| f.transpose())
    }
}

impl FromRequest for Multipart {
    type Error = MultipartError;

    async fn from_request(req: &mut Request) -> Result<Self, Self::Error> {
        let Some(boundary) = Self::parse_boundary(req.headers()) else {
            return Err(MultipartError::UnsupportedContentType);
        };

        let stream = std::mem::take(req.body_mut()).into_data_stream();

        Ok(Self {
            inner: multer::Multipart::new(stream, boundary),
        })
    }
}

/// 多部分流中的单个字段。
#[derive(Debug)]
pub struct Field {
    inner: multer::Field<'static>,
}

impl Field {
    /// 在 `Content-Disposition` 标头中找到的字段名。
    pub fn name(&self) -> Option<&str> {
        self.inner.name()
    }

    /// 在 `Content-Disposition` 标头中找到的文件名。
    pub fn file_name(&self) -> Option<&str> {
        self.inner.file_name()
    }

    /// 获取字段的内容类型。
    pub fn content_type(&self) -> Option<&str> {
        self.inner.content_type().map(|m| m.as_ref())
    }

    /// 获取字段的标头集。
    pub fn headers(&self) -> &HeaderMap {
        self.inner.headers()
    }

    /// 以二进制的形式获取字段的完整数据。
    pub async fn bytes(self) -> Result<Bytes, MultipartError> {
        self.inner
            .bytes()
            .await
            .map_err(MultipartError::from_multer)
    }

    /// 以文本的形式获取完整的字段数据。
    pub async fn text(self) -> Result<String, MultipartError> {
        self.inner.text().await.map_err(MultipartError::from_multer)
    }

    /// 流式获取字段的数据块。
    pub async fn chunk(&mut self) -> Result<Option<Bytes>, MultipartError> {
        self.inner
            .chunk()
            .await
            .map_err(MultipartError::from_multer)
    }
}

impl Stream for Field {
    type Item = Result<Bytes, MultipartError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.inner)
            .poll_next(cx)
            .map_err(MultipartError::from_multer)
    }
}

/// `multipart/form-data` 解析错误。
#[derive(Debug, Clone)]
pub enum MultipartError {
    /// 不支持的内容类型。
    UnsupportedContentType,
    /// 解析错误。
    ParseError(String),
}

impl MultipartError {
    fn from_multer(error: multer::Error) -> Self {
        MultipartError::ParseError(error.to_string())
    }
}

impl std::fmt::Display for MultipartError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MultipartError::UnsupportedContentType => f.write_str("unsupported content type"),
            MultipartError::ParseError(e) => {
                write!(f, "failed to parse `multipart/form-data` request ({e})")
            }
        }
    }
}

impl std::error::Error for MultipartError {}
