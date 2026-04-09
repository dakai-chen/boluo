use std::pin::Pin;
use std::time::Duration;

use tokio::sync::watch::{self, Receiver, Sender};

/// 优雅关机，用于等待服务器完成剩余请求。
#[derive(Debug)]
pub struct GracefulShutdown {
    tx: Sender<()>,
    rx: Receiver<()>,
}

#[derive(Debug, Clone)]
pub(super) struct Monitor {
    rx: Receiver<()>,
}

impl Monitor {
    /// 监视任务并在接收到关机信号时执行关机操作。
    pub(super) async fn watch<T>(mut self, task: T, shutdown: impl FnOnce(Pin<&mut T>)) -> T::Output
    where
        T: Future,
    {
        let mut task = std::pin::pin!(task);
        tokio::select! {
            _ = self.rx.changed() => {
                shutdown(task.as_mut());
                task.await
            }
            v = task.as_mut() => v
        }
    }
}

impl GracefulShutdown {
    /// 创建新的 `GracefulShutdown` 实例。
    pub(super) fn new() -> Self {
        let (tx, rx) = watch::channel(());
        Self { tx, rx }
    }

    /// 创建一个 `Monitor` 实例，用于监视任务。
    pub(super) fn monitor(&self) -> Monitor {
        Monitor {
            rx: self.rx.clone(),
        }
    }

    /// 发出关机信号，在指定时间内等待服务器完成剩余请求。
    ///
    /// - 返回 Ok：表示服务器完成了所有剩余请求。
    /// - 返回 Err：表示在指定时间内服务器未能完成所有剩余请求。
    pub async fn shutdown(self, timeout: Option<Duration>) -> Result<(), GracefulShutdownTimeout> {
        let GracefulShutdown { tx, rx } = self;

        drop(rx);
        tx.send_modify(|_| {});

        tokio::select! {
            _ = tx.closed() => Ok(()),
            _ = sleep(timeout) => Err(GracefulShutdownTimeout { _priv: () }),
        }
    }
}

async fn sleep(timeout: Option<Duration>) {
    if let Some(timeout) = timeout {
        tokio::time::sleep(timeout).await
    } else {
        std::future::pending().await
    }
}

/// 优雅关机超时。
pub struct GracefulShutdownTimeout {
    _priv: (),
}

impl std::fmt::Debug for GracefulShutdownTimeout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GracefulShutdownTimeout").finish()
    }
}

impl std::fmt::Display for GracefulShutdownTimeout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "server graceful shutdown timeout")
    }
}

impl std::error::Error for GracefulShutdownTimeout {}
