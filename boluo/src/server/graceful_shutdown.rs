use std::pin::Pin;
use std::time::Duration;

use tokio::sync::watch::{self, Receiver, Sender};
use tokio_util::sync::CancellationToken;

/// 优雅关机，用于等待服务器完成剩余请求。
#[derive(Debug)]
pub struct GracefulShutdown {
    tx: Sender<()>,
    rx: Receiver<()>,
    shutdown_signal: CancellationToken,
}

#[derive(Debug, Clone)]
pub(super) struct Monitor {
    rx: Receiver<()>,
    shutdown_signal: CancellationToken,
}

impl Monitor {
    /// 监视任务并在接收到关机信号时执行关机操作。
    pub(super) async fn watch<T>(self, task: T, shutdown: impl FnOnce(Pin<&mut T>)) -> T::Output
    where
        T: Future,
    {
        let _rx = self.rx;
        let mut task = std::pin::pin!(task);

        tokio::select! {
            _ = self.shutdown_signal.cancelled() => {
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
        Self {
            tx,
            rx,
            shutdown_signal: CancellationToken::new(),
        }
    }

    /// 创建一个 `Monitor` 实例，用于监视任务。
    pub(super) fn monitor(&self) -> Monitor {
        Monitor {
            rx: self.rx.clone(),
            shutdown_signal: self.shutdown_signal.clone(),
        }
    }

    /// 发出关机信号，在指定时间内等待服务器完成剩余请求。
    ///
    /// 如果超时时间设置为 `None`，则该函数将持续等待，直到服务器完成所有剩余请求为止。
    /// 如果设置了超时时间，则该函数将在超时时间到达时返回。
    ///
    /// 该函数返回 `true` 表示服务器完成了所有剩余请求，
    /// 返回 `false` 表示在超时时间内服务器未能完成所有剩余请求。
    pub async fn shutdown(self, timeout: Option<Duration>) -> bool {
        let GracefulShutdown {
            tx,
            rx,
            shutdown_signal,
        } = self;

        drop(rx);
        shutdown_signal.cancel();

        tokio::select! {
            _ = tx.closed() => true,
            _ = sleep(timeout) => false,
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
