//! HTTP 升级。

use std::any::Any;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};

use futures_core::future::BoxFuture;
use futures_io::{AsyncRead, AsyncWrite};

use crate::BoxError;

type UpgradeFuture = BoxFuture<'static, Result<Upgraded, BoxError>>;

/// 用于处理 HTTP 升级请求，获取升级后的连接。
#[derive(Clone)]
pub struct OnUpgrade {
    fut: Arc<Mutex<Option<UpgradeFuture>>>,
}

impl OnUpgrade {
    /// 创建一个 `OnUpgrade` 实例。
    pub fn new<T>(fut: T) -> Self
    where
        T: Future<Output = Result<Upgraded, BoxError>> + Send + 'static,
    {
        Self {
            fut: Arc::new(Mutex::new(Some(Box::pin(fut)))),
        }
    }
}

impl Future for OnUpgrade {
    type Output = Result<Upgraded, OnUpgradeError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut guard = self.fut.lock().unwrap();

        let Some(fut) = guard.as_mut() else {
            return Poll::Ready(Err(OnUpgradeError::Consumed));
        };

        let poll = fut.as_mut().poll(cx);

        if poll.is_ready() {
            guard.take();
        }

        poll.map_err(OnUpgradeError::Failed)
    }
}

impl std::fmt::Debug for OnUpgrade {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OnUpgrade").finish()
    }
}

/// HTTP 升级后的连接。
pub struct Upgraded {
    io: Box<dyn IO + Send>,
}

impl Upgraded {
    /// 创建一个 `Upgraded` 实例。
    pub fn new<T>(io: T) -> Self
    where
        T: AsyncRead + AsyncWrite + Unpin + Send + 'static,
    {
        Self { io: Box::new(io) }
    }

    /// 尝试将 `Upgraded` 实例转换为指定类型。
    pub fn downcast<T: 'static>(self) -> Result<T, Self> {
        if self.io.as_ref().as_any().is::<T>() {
            Ok(*self.io.into_any().downcast::<T>().unwrap())
        } else {
            Err(self)
        }
    }
}

impl AsyncRead for Upgraded {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<std::io::Result<usize>> {
        Pin::new(&mut self.io).poll_read(cx, buf)
    }
}

impl AsyncWrite for Upgraded {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        Pin::new(&mut self.io).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.io).poll_flush(cx)
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.io).poll_close(cx)
    }
}

impl std::fmt::Debug for Upgraded {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Upgraded").finish()
    }
}

trait IO: AsyncRead + AsyncWrite + Unpin + 'static {
    fn as_any(&self) -> &dyn Any;

    fn into_any(self: Box<Self>) -> Box<dyn Any>;
}

impl<T: AsyncRead + AsyncWrite + Unpin + 'static> IO for T {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}

/// HTTP 协议升级错误。
#[derive(Debug)]
pub enum OnUpgradeError {
    /// 升级任务已被消费。
    Consumed,
    /// 协议升级过程失败。
    Failed(BoxError),
}

impl std::fmt::Display for OnUpgradeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OnUpgradeError::Consumed => write!(f, "http upgrade future has already been consumed"),
            OnUpgradeError::Failed(e) => write!(f, "http upgrade failed: {e}"),
        }
    }
}

impl std::error::Error for OnUpgradeError {}

#[cfg(test)]
mod tests {
    use std::pin::Pin;
    use std::task::{Context, Poll};

    use futures_io::{AsyncRead, AsyncWrite};

    use super::Upgraded;

    struct FuturesIo;

    impl AsyncRead for FuturesIo {
        fn poll_read(
            self: Pin<&mut Self>,
            _cx: &mut Context<'_>,
            _buf: &mut [u8],
        ) -> Poll<std::io::Result<usize>> {
            todo!()
        }
    }

    impl AsyncWrite for FuturesIo {
        fn poll_write(
            self: Pin<&mut Self>,
            _cx: &mut Context<'_>,
            _buf: &[u8],
        ) -> Poll<std::io::Result<usize>> {
            todo!()
        }

        fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
            todo!()
        }

        fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
            todo!()
        }
    }

    #[test]
    fn upgraded_downcast() {
        assert!(Upgraded::new(FuturesIo).downcast::<()>().is_err());
        assert!(Upgraded::new(FuturesIo).downcast::<FuturesIo>().is_ok());
    }
}
