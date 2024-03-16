use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use boluo_core::body::Bytes;
use boluo_core::extract::FromRequest;
use boluo_core::http::header::CONTENT_TYPE;
use boluo_core::http::HeaderMap;
use boluo_core::request::Request;
use boluo_core::BoxError;
use futures_util::Stream;

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
        let boundary =
            Self::parse_boundary(req.headers()).ok_or(MultipartError::UnsupportedContentType)?;

        let stream = std::mem::take(req.body_mut()).into_data_stream();

        Ok(Self {
            inner: multer::Multipart::new(stream, boundary),
        })
    }
}

#[derive(Debug)]
pub struct Field {
    inner: multer::Field<'static>,
}

impl Field {
    pub fn name(&self) -> Option<&str> {
        self.inner.name()
    }

    pub fn file_name(&self) -> Option<&str> {
        self.inner.file_name()
    }

    pub fn content_type(&self) -> Option<&str> {
        self.inner.content_type().map(|m| m.as_ref())
    }

    pub fn headers(&self) -> &HeaderMap {
        self.inner.headers()
    }

    pub async fn bytes(self) -> Result<Bytes, MultipartError> {
        self.inner
            .bytes()
            .await
            .map_err(MultipartError::from_multer)
    }

    pub async fn text(self) -> Result<String, MultipartError> {
        self.inner.text().await.map_err(MultipartError::from_multer)
    }

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

#[derive(Debug)]
pub enum MultipartError {
    UnsupportedContentType,
    Other(BoxError),
}

impl MultipartError {
    fn from_multer(error: multer::Error) -> Self {
        MultipartError::Other(error.into())
    }
}

impl std::fmt::Display for MultipartError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MultipartError::UnsupportedContentType => f.write_str("unsupported content type"),
            MultipartError::Other(e) => {
                write!(f, "error parsing `multipart/form-data` request ({e})")
            }
        }
    }
}

impl std::error::Error for MultipartError {}
