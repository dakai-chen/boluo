//! HTTP 主体。

pub use bytes::Bytes;
pub use http_body::{Body as HttpBody, Frame, SizeHint};

use std::borrow::Cow;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures_core::{Stream, TryStream};
use http_body_util::{BodyExt, Empty, Full};

use crate::BoxError;

type BoxBody = http_body_util::combinators::UnsyncBoxBody<Bytes, BoxError>;

fn boxed<B>(body: B) -> BoxBody
where
    B: HttpBody<Data = Bytes> + Send + 'static,
    B::Error: Into<BoxError>,
{
    crate::util::__try_downcast(body).unwrap_or_else(|body| body.map_err(Into::into).boxed_unsync())
}

/// 请求和响应的主体类型。
#[derive(Debug)]
pub struct Body(BoxBody);

impl Body {
    /// 使用给定的 [`HttpBody`] 对象，创建一个新的 [`Body`]。
    pub fn new<B>(body: B) -> Self
    where
        B: HttpBody<Data = Bytes> + Send + 'static,
        B::Error: Into<BoxError>,
    {
        crate::util::__try_downcast(body).unwrap_or_else(|body| Self(boxed(body)))
    }

    /// 创建一个空的 [`Body`]。
    pub fn empty() -> Self {
        Self::new(Empty::new())
    }

    /// 从 [`Stream`] 中创建一个新的 [`Body`]。
    pub fn from_data_stream<S>(stream: S) -> Self
    where
        S: TryStream + Send + 'static,
        S::Ok: Into<Bytes>,
        S::Error: Into<BoxError>,
    {
        Self::new(StreamBody { stream })
    }

    /// 将 [`Body`] 的数据帧转换为 [`Stream`]，非数据帧的部分将被丢弃。
    pub fn into_data_stream(self) -> BodyDataStream {
        BodyDataStream { inner: self }
    }

    /// 消耗此 [`Body`] 对象，将其所有数据收集并合并为单个 [`Bytes`] 缓冲区。
    pub async fn to_bytes(self) -> Result<Bytes, BoxError> {
        self.collect().await.map(|col| col.to_bytes())
    }
}

impl Default for Body {
    fn default() -> Self {
        Self::empty()
    }
}

impl From<()> for Body {
    fn from(_: ()) -> Self {
        Self::empty()
    }
}

macro_rules! body_from_impl {
    ($ty:ty) => {
        impl From<$ty> for Body {
            fn from(buf: $ty) -> Self {
                Self::new(Full::from(buf))
            }
        }
    };
}

body_from_impl!(&'static [u8]);
body_from_impl!(Cow<'static, [u8]>);
body_from_impl!(Vec<u8>);

body_from_impl!(&'static str);
body_from_impl!(Cow<'static, str>);
body_from_impl!(String);

body_from_impl!(Bytes);

impl HttpBody for Body {
    type Data = Bytes;
    type Error = BoxError;

    #[inline]
    fn poll_frame(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        Pin::new(&mut self.0).poll_frame(cx)
    }

    #[inline]
    fn size_hint(&self) -> SizeHint {
        self.0.size_hint()
    }

    #[inline]
    fn is_end_stream(&self) -> bool {
        self.0.is_end_stream()
    }
}

/// [`Body`] 的数据帧流。
#[derive(Debug)]
pub struct BodyDataStream {
    inner: Body,
}

impl Stream for BodyDataStream {
    type Item = Result<Bytes, BoxError>;

    #[inline]
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            match futures_core::ready!(Pin::new(&mut self.inner).poll_frame(cx)?) {
                Some(frame) => match frame.into_data() {
                    Ok(data) => return Poll::Ready(Some(Ok(data))),
                    Err(_frame) => {}
                },
                None => return Poll::Ready(None),
            }
        }
    }
}

impl HttpBody for BodyDataStream {
    type Data = Bytes;
    type Error = BoxError;

    #[inline]
    fn poll_frame(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        Pin::new(&mut self.inner).poll_frame(cx)
    }

    #[inline]
    fn is_end_stream(&self) -> bool {
        self.inner.is_end_stream()
    }

    #[inline]
    fn size_hint(&self) -> SizeHint {
        self.inner.size_hint()
    }
}

pin_project_lite::pin_project! {
    struct StreamBody<S> {
        #[pin]
        stream: S,
    }
}

impl<S> HttpBody for StreamBody<S>
where
    S: TryStream,
    S::Ok: Into<Bytes>,
    S::Error: Into<BoxError>,
{
    type Data = Bytes;
    type Error = BoxError;

    fn poll_frame(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        let this = self.project();
        match futures_core::ready!(this.stream.try_poll_next(cx)) {
            Some(Ok(chunk)) => Poll::Ready(Some(Ok(Frame::data(chunk.into())))),
            Some(Err(err)) => Poll::Ready(Some(Err(err.into()))),
            None => Poll::Ready(None),
        }
    }
}
