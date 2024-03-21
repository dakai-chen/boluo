use std::borrow::Cow;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

use futures_util::ready;
use tokio::time::{Instant, Sleep};

use super::{Event, EventValueError};

/// 用于配置保持连接的消息间隔和消息文本。
#[derive(Debug, Clone)]
pub struct KeepAlive {
    event: Event,
    interval: Duration,
}

impl KeepAlive {
    /// 创建一个新的[`KeepAlive`]。
    pub fn new() -> Self {
        Default::default()
    }

    /// 自定义保持连接的消息文本。
    ///
    /// 默认为空注释。
    pub fn text(mut self, text: impl Into<Cow<'static, str>>) -> Result<Self, EventValueError> {
        Event::builder().comment(text).build().map(|event| {
            self.event = event;
            self
        })
    }

    /// 自定义保持连接的消息间隔。
    ///
    /// 默认值为15秒。
    pub fn interval(mut self, time: Duration) -> Self {
        self.interval = time;
        self
    }
}

impl Default for KeepAlive {
    fn default() -> Self {
        Self {
            event: Event::builder().comment("").build().unwrap(),
            interval: Duration::from_secs(15),
        }
    }
}

pin_project_lite::pin_project! {
    pub(super) struct KeepAliveStream {
        keep_alive: KeepAlive,
        #[pin]
        timer: Sleep,
    }
}

impl KeepAliveStream {
    pub(super) fn new(keep_alive: KeepAlive) -> Self {
        Self {
            timer: tokio::time::sleep(keep_alive.interval),
            keep_alive,
        }
    }

    pub(super) fn reset(self: Pin<&mut Self>) {
        let this = self.project();
        this.timer.reset(Instant::now() + this.keep_alive.interval);
    }

    pub(super) fn poll_event(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Event> {
        ready!(self.as_mut().project().timer.poll(cx));

        self.as_mut().reset();

        Poll::Ready(self.keep_alive.event.clone())
    }
}
