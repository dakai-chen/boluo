use std::pin::Pin;
use std::{future::Future, time::Duration};

use tokio::sync::mpsc::{self, Receiver, Sender};
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
    tx: Sender<()>,
    shutdown_signal: CancellationToken,
}

impl Monitor {
    pub(super) async fn watch<T>(self, task: T, shutdown: impl FnOnce(Pin<&mut T>)) -> T::Output
    where
        T: Future,
    {
        let _tx = self.tx;
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
    pub(super) fn new() -> Self {
        let (tx, rx) = mpsc::channel::<()>(1);
        Self {
            tx,
            rx,
            shutdown_signal: CancellationToken::new(),
        }
    }

    pub(super) fn monitor(&self) -> Monitor {
        Monitor {
            tx: self.tx.clone(),
            shutdown_signal: self.shutdown_signal.clone(),
        }
    }

    /// 发出关机信号，在指定时间内等待服务器完成剩余请求。若剩余请求在指定时间内完成，则提前返回，
    /// 返回值为`true`，超时则返回`false`。
    ///
    /// 未设置指定时间将一直等待，直到服务器完成所有请求。
    pub async fn shutdown(self, timeout: Option<Duration>) -> bool {
        let GracefulShutdown {
            tx,
            mut rx,
            shutdown_signal,
        } = self;

        drop(tx);
        shutdown_signal.cancel();

        tokio::select! {
            _ = rx.recv() => true,
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
