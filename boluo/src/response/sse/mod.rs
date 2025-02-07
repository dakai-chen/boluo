//! 服务器发送事件（SSE）。

mod event;
pub use event::{Event, EventBuilder, EventValueError};

mod keep_alive;
pub use keep_alive::KeepAlive;

use std::convert::Infallible;
use std::pin::Pin;
use std::task::{Context, Poll};

use boluo_core::BoxError;
use boluo_core::body::{Body, Bytes, Frame, HttpBody};
use boluo_core::http::header;
use boluo_core::response::{IntoResponse, Response};
use futures_util::Stream;

use self::keep_alive::KeepAliveStream;

/// 服务器发送事件的响应。
#[derive(Clone)]
pub struct Sse<S> {
    stream: S,
    keep_alive: Option<KeepAlive>,
}

impl<S> Sse<S> {
    /// 创建一个新的[`Sse`]响应。
    pub fn new(stream: S) -> Self {
        Self {
            stream,
            keep_alive: None,
        }
    }

    /// 保持连接。
    pub fn keep_alive(mut self, keep_alive: KeepAlive) -> Self {
        self.keep_alive = Some(keep_alive);
        self
    }
}

impl<S> std::fmt::Debug for Sse<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Sse")
            .field("stream", &std::any::type_name::<S>())
            .field("keep_alive", &self.keep_alive)
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
            keep_alive: self.keep_alive.map(KeepAliveStream::new),
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
        #[pin]
        keep_alive: Option<KeepAliveStream>,
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

        match this.stream.poll_next(cx) {
            Poll::Pending => {
                if let Some(keep_alive) = this.keep_alive.as_pin_mut() {
                    keep_alive
                        .poll_event(cx)
                        .map(|e| Some(Ok(Frame::data(Bytes::from(e.to_string())))))
                } else {
                    Poll::Pending
                }
            }
            Poll::Ready(Some(Ok(event))) => {
                if let Some(keep_alive) = this.keep_alive.as_pin_mut() {
                    keep_alive.reset();
                }
                Poll::Ready(Some(Ok(Frame::data(Bytes::from(event.to_string())))))
            }
            Poll::Ready(Some(Err(error))) => Poll::Ready(Some(Err(error))),
            Poll::Ready(None) => Poll::Ready(None),
        }
    }
}
