//! 服务器发送事件（SSE）。

mod event;
#[cfg(feature = "tokio")]
mod keep_alive;

pub use event::{Event, EventBuilder, EventValueError};
#[cfg(feature = "tokio")]
pub use keep_alive::{KeepAlive, KeepAliveStream};

use std::convert::Infallible;
use std::pin::Pin;
use std::task::{Context, Poll};

use boluo_core::BoxError;
use boluo_core::body::{Body, Bytes, Frame, HttpBody};
use boluo_core::http::header;
use boluo_core::response::{IntoResponse, Response};
use futures_util::Stream;

/// 服务器发送事件。
#[derive(Clone)]
pub struct Sse<S> {
    stream: S,
}

impl<S> Sse<S> {
    /// 创建一个新的 [`Sse`] 实例。
    pub fn new(stream: S) -> Self {
        Self { stream }
    }

    /// 保持连接。
    #[cfg(feature = "tokio")]
    pub fn keep_alive(self, keep_alive: KeepAlive) -> Sse<KeepAliveStream<S>> {
        Sse {
            stream: KeepAliveStream::new(keep_alive, self.stream),
        }
    }
}

impl<S> std::fmt::Debug for Sse<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Sse")
            .field("stream", &std::any::type_name::<S>())
            .finish()
    }
}

impl<S, E> IntoResponse for Sse<S>
where
    S: Stream<Item = Result<Event, E>> + Send + 'static,
    E: Into<BoxError>,
{
    type Error = Infallible;

    fn into_response(self) -> Result<Response, Self::Error> {
        let body = SseBody {
            stream: self.stream,
        };

        Response::builder()
            .header(header::CONTENT_TYPE, mime::TEXT_EVENT_STREAM.as_ref())
            .header(header::CACHE_CONTROL, "no-cache")
            .body(Body::new(body))
            .map_err(|e| unreachable!("{e}"))
    }
}

pin_project_lite::pin_project! {
    struct SseBody<S> {
        #[pin]
        stream: S,
    }
}

impl<S, E> HttpBody for SseBody<S>
where
    S: Stream<Item = Result<Event, E>>,
{
    type Data = Bytes;
    type Error = E;

    fn poll_frame(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Bytes>, Self::Error>>> {
        let this = self.project();

        match std::task::ready!(this.stream.poll_next(cx)) {
            Some(Ok(event)) => Poll::Ready(Some(Ok(Frame::data(Bytes::from(event.to_string()))))),
            Some(Err(error)) => Poll::Ready(Some(Err(error))),
            None => Poll::Ready(None),
        }
    }
}
