//! HTTP升级。

use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};

use futures_core::future::BoxFuture;
use futures_io::{AsyncRead, AsyncWrite};

use crate::BoxError;

/// 用于处理 HTTP 升级请求，获取升级后的连接。
#[derive(Clone)]
pub struct OnUpgrade {
    fut: Arc<Mutex<BoxFuture<'static, Result<Upgraded, BoxError>>>>,
}

impl OnUpgrade {
    /// 创建一个 `OnUpgrade` 实例。
    pub fn new<T>(fut: T) -> Self
    where
        T: Future<Output = Result<Upgraded, BoxError>> + Send + 'static,
    {
        Self {
            fut: Arc::new(Mutex::new(Box::pin(fut))),
        }
    }
}

impl Future for OnUpgrade {
    type Output = Result<Upgraded, BoxError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.fut.lock().unwrap().as_mut().poll(cx)
    }
}

impl std::fmt::Debug for OnUpgrade {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OnUpgrade").finish()
    }
}

/// HTTP 升级后的连接。
pub struct Upgraded {
    inner: Box<dyn IO + Send + 'static>,
}

impl Upgraded {
    /// 创建一个 `Upgraded` 实例。
    pub fn new<T>(inner: T) -> Self
    where
        T: AsyncRead + AsyncWrite + Unpin + Send + 'static,
    {
        Self {
            inner: Box::new(inner),
        }
    }

    /// 尝试将 `Upgraded` 实例转换为指定类型。
    pub fn downcast<T: 'static>(self) -> Result<T, Self> {
        crate::util::__try_downcast(self)
    }
}

impl AsyncRead for Upgraded {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<std::io::Result<usize>> {
        Pin::new(&mut self.inner).poll_read(cx, buf)
    }
}

impl AsyncWrite for Upgraded {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        Pin::new(&mut self.inner).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.inner).poll_flush(cx)
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.inner).poll_close(cx)
    }
}

impl std::fmt::Debug for Upgraded {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Upgraded").finish()
    }
}

trait IO: AsyncRead + AsyncWrite + Unpin {}

impl<T: ?Sized + AsyncRead + AsyncWrite + Unpin> IO for T {}
