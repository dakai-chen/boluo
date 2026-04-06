use std::borrow::Cow;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

use futures_util::Stream;
use tokio::time::{Instant, Sleep};

use super::Event;

/// 用于配置保持连接的消息间隔和消息文本。
#[derive(Debug, Clone)]
pub struct KeepAlive {
    event: Event,
    interval: Duration,
}

impl KeepAlive {
    /// 创建一个新的 [`KeepAlive`] 实例。
    pub fn new() -> Self {
        Default::default()
    }

    /// 自定义保持连接的消息间隔。
    ///
    /// 默认值为 15 秒。
    pub fn interval(mut self, time: Duration) -> Self {
        self.interval = time;
        self
    }

    /// 自定义保持连接的消息文本。
    ///
    /// 默认为空注释。
    ///
    /// # 恐慌
    ///
    /// 如果设置的消息文本包含换行符或回车符。
    pub fn text(self, text: impl Into<Cow<'static, str>>) -> Self {
        self.event(Event::default().comment(text))
    }

    /// 自定义保持连接的消息事件。
    ///
    /// 默认为空注释。
    pub fn event(mut self, event: Event) -> Self {
        self.event = event;
        self
    }
}

impl Default for KeepAlive {
    fn default() -> Self {
        Self {
            event: Event::default().comment(""),
            interval: Duration::from_secs(15),
        }
    }
}

pin_project_lite::pin_project! {
    /// 对流的包装，用于自动产生保持连接的心跳事件。
    pub struct KeepAliveStream<S> {
        keep_alive: KeepAlive,
        #[pin]
        timer: Sleep,
        #[pin]
        inner: S,
    }
}

impl<S> KeepAliveStream<S> {
    pub(super) fn new(keep_alive: KeepAlive, inner: S) -> Self {
        Self {
            timer: tokio::time::sleep(keep_alive.interval),
            keep_alive,
            inner,
        }
    }

    pub(super) fn reset(self: Pin<&mut Self>) {
        let this = self.project();
        this.timer.reset(Instant::now() + this.keep_alive.interval);
    }
}

impl<S, E> Stream for KeepAliveStream<S>
where
    S: Stream<Item = Result<Event, E>>,
{
    type Item = Result<Event, E>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.as_mut().project();

        match this.inner.as_mut().poll_next(cx) {
            Poll::Ready(Some(Ok(event))) => {
                self.reset();

                Poll::Ready(Some(Ok(event)))
            }
            Poll::Ready(Some(Err(error))) => Poll::Ready(Some(Err(error))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => {
                std::task::ready!(this.timer.poll(cx));

                let event = this.keep_alive.event.clone();

                self.reset();

                Poll::Ready(Some(Ok(event)))
            }
        }
    }
}
