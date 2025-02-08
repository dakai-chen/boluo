//! HTTP服务器。

mod compat;

mod graceful_shutdown;
pub use graceful_shutdown::GracefulShutdown;

use std::convert::Infallible;
use std::future::Future;
use std::time::Duration;

use boluo_core::BoxError;
use boluo_core::http::StatusCode;
use boluo_core::request::Request;
use boluo_core::response::{IntoResponse, Response};
use boluo_core::service::{ArcService, Service, ServiceExt};
use hyper::service::Service as _;
use hyper_util::rt::{TokioExecutor, TokioIo, TokioTimer};
use hyper_util::server::conn::auto::Builder;
use tokio::io::{AsyncRead, AsyncWrite};

use crate::listener::Listener;

/// HTTP服务器。
pub struct Server<L> {
    listener: L,
    builder: Builder<TokioExecutor>,
}

impl<L> std::fmt::Debug for Server<L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Server")
            .field("listener", &std::any::type_name::<L>())
            .field("builder", &self.builder)
            .finish()
    }
}

impl<L> Server<L>
where
    L: Listener,
    L::IO: AsyncRead + AsyncWrite + Unpin + Send + 'static,
    L::Addr: Clone + Send + Sync + 'static,
{
    /// 使用指定的监听器创建服务器。
    pub fn new(listener: L) -> Self {
        let mut builder = Builder::new(TokioExecutor::new());
        #[cfg(feature = "http1")]
        {
            builder.http1().timer(TokioTimer::default());
        }
        #[cfg(feature = "http2")]
        {
            builder.http2().timer(TokioTimer::default());
        }
        Self { listener, builder }
    }

    /// See [`Http1Builder::half_close`]。
    ///
    /// [`Http1Builder::half_close`]: hyper_util::server::conn::auto::Http1Builder::half_close
    #[cfg(feature = "http1")]
    pub fn http1_half_close(&mut self, val: bool) -> &mut Self {
        self.builder.http1().half_close(val);
        self
    }

    /// See [`Http1Builder::keep_alive`]。
    ///
    /// [`Http1Builder::keep_alive`]: hyper_util::server::conn::auto::Http1Builder::keep_alive
    #[cfg(feature = "http1")]
    pub fn http1_keep_alive(&mut self, val: bool) -> &mut Self {
        self.builder.http1().keep_alive(val);
        self
    }

    /// See [`Http1Builder::title_case_headers`]。
    ///
    /// [`Http1Builder::title_case_headers`]: hyper_util::server::conn::auto::Http1Builder::title_case_headers
    #[cfg(feature = "http1")]
    pub fn http1_title_case_headers(&mut self, enabled: bool) -> &mut Self {
        self.builder.http1().title_case_headers(enabled);
        self
    }

    /// See [`Http1Builder::preserve_header_case`]。
    ///
    /// [`Http1Builder::preserve_header_case`]: hyper_util::server::conn::auto::Http1Builder::preserve_header_case
    #[cfg(feature = "http1")]
    pub fn http1_preserve_header_case(&mut self, enabled: bool) -> &mut Self {
        self.builder.http1().preserve_header_case(enabled);
        self
    }

    /// See [`Http1Builder::max_headers`]。
    ///
    /// [`Http1Builder::max_headers`]: hyper_util::server::conn::auto::Http1Builder::max_headers
    #[cfg(feature = "http1")]
    pub fn http1_max_headers(&mut self, val: usize) -> &mut Self {
        self.builder.http1().max_headers(val);
        self
    }

    /// See [`Http1Builder::header_read_timeout`]。
    ///
    /// [`Http1Builder::header_read_timeout`]: hyper_util::server::conn::auto::Http1Builder::header_read_timeout
    #[cfg(feature = "http1")]
    pub fn http1_header_read_timeout(
        &mut self,
        read_timeout: impl Into<Option<Duration>>,
    ) -> &mut Self {
        self.builder.http1().header_read_timeout(read_timeout);
        self
    }

    /// See [`Http1Builder::writev`]。
    ///
    /// [`Http1Builder::writev`]: hyper_util::server::conn::auto::Http1Builder::writev
    #[cfg(feature = "http1")]
    pub fn http1_writev(&mut self, val: bool) -> &mut Self {
        self.builder.http1().writev(val);
        self
    }

    /// See [`Http1Builder::max_buf_size`]。
    ///
    /// [`Http1Builder::max_buf_size`]: hyper_util::server::conn::auto::Http1Builder::max_buf_size
    #[cfg(feature = "http1")]
    pub fn http1_max_buf_size(&mut self, max: usize) -> &mut Self {
        self.builder.http1().max_buf_size(max);
        self
    }

    /// See [`Http1Builder::pipeline_flush`]。
    ///
    /// [`Http1Builder::pipeline_flush`]: hyper_util::server::conn::auto::Http1Builder::pipeline_flush
    #[cfg(feature = "http1")]
    pub fn http1_pipeline_flush(&mut self, enabled: bool) -> &mut Self {
        self.builder.http1().pipeline_flush(enabled);
        self
    }

    /// See [`Http2Builder::max_pending_accept_reset_streams`]。
    ///
    /// [`Http2Builder::max_pending_accept_reset_streams`]: hyper_util::server::conn::auto::Http2Builder::max_pending_accept_reset_streams
    #[cfg(feature = "http2")]
    pub fn http2_max_pending_accept_reset_streams(
        &mut self,
        max: impl Into<Option<usize>>,
    ) -> &mut Self {
        self.builder.http2().max_pending_accept_reset_streams(max);
        self
    }

    /// See [`Http2Builder::initial_stream_window_size`]。
    ///
    /// [`Http2Builder::initial_stream_window_size`]: hyper_util::server::conn::auto::Http2Builder::initial_stream_window_size
    #[cfg(feature = "http2")]
    pub fn http2_initial_stream_window_size(&mut self, sz: impl Into<Option<u32>>) -> &mut Self {
        self.builder.http2().initial_stream_window_size(sz);
        self
    }

    /// See [`Http2Builder::initial_connection_window_size`]。
    ///
    /// [`Http2Builder::initial_connection_window_size`]: hyper_util::server::conn::auto::Http2Builder::initial_connection_window_size
    #[cfg(feature = "http2")]
    pub fn http2_initial_connection_window_size(
        &mut self,
        sz: impl Into<Option<u32>>,
    ) -> &mut Self {
        self.builder.http2().initial_connection_window_size(sz);
        self
    }

    /// See [`Http2Builder::adaptive_window`]。
    ///
    /// [`Http2Builder::adaptive_window`]: hyper_util::server::conn::auto::Http2Builder::adaptive_window
    #[cfg(feature = "http2")]
    pub fn http2_adaptive_window(&mut self, enabled: bool) -> &mut Self {
        self.builder.http2().adaptive_window(enabled);
        self
    }

    /// See [`Http2Builder::max_frame_size`]。
    ///
    /// [`Http2Builder::max_frame_size`]: hyper_util::server::conn::auto::Http2Builder::max_frame_size
    #[cfg(feature = "http2")]
    pub fn http2_max_frame_size(&mut self, sz: impl Into<Option<u32>>) -> &mut Self {
        self.builder.http2().max_frame_size(sz);
        self
    }

    /// See [`Http2Builder::max_concurrent_streams`]。
    ///
    /// [`Http2Builder::max_concurrent_streams`]: hyper_util::server::conn::auto::Http2Builder::max_concurrent_streams
    #[cfg(feature = "http2")]
    pub fn http2_max_concurrent_streams(&mut self, max: impl Into<Option<u32>>) -> &mut Self {
        self.builder.http2().max_concurrent_streams(max);
        self
    }

    /// See [`Http2Builder::keep_alive_interval`]。
    ///
    /// [`Http2Builder::keep_alive_interval`]: hyper_util::server::conn::auto::Http2Builder::keep_alive_interval
    #[cfg(feature = "http2")]
    pub fn http2_keep_alive_interval(
        &mut self,
        interval: impl Into<Option<Duration>>,
    ) -> &mut Self {
        self.builder.http2().keep_alive_interval(interval);
        self
    }

    /// See [`Http2Builder::keep_alive_timeout`]。
    ///
    /// [`Http2Builder::keep_alive_timeout`]: hyper_util::server::conn::auto::Http2Builder::keep_alive_timeout
    #[cfg(feature = "http2")]
    pub fn http2_keep_alive_timeout(&mut self, timeout: Duration) -> &mut Self {
        self.builder.http2().keep_alive_timeout(timeout);
        self
    }

    /// See [`Http2Builder::max_send_buf_size`]。
    ///
    /// [`Http2Builder::max_send_buf_size`]: hyper_util::server::conn::auto::Http2Builder::max_send_buf_size
    #[cfg(feature = "http2")]
    pub fn http2_max_send_buf_size(&mut self, max: usize) -> &mut Self {
        self.builder.http2().max_send_buf_size(max);
        self
    }

    /// See [`Http2Builder::enable_connect_protocol`]。
    ///
    /// [`Http2Builder::enable_connect_protocol`]: hyper_util::server::conn::auto::Http2Builder::enable_connect_protocol
    #[cfg(feature = "http2")]
    pub fn http2_enable_connect_protocol(&mut self) -> &mut Self {
        self.builder.http2().enable_connect_protocol();
        self
    }

    /// See [`Http2Builder::max_header_list_size`]。
    ///
    /// [`Http2Builder::max_header_list_size`]: hyper_util::server::conn::auto::Http2Builder::max_header_list_size
    #[cfg(feature = "http2")]
    pub fn http2_max_header_list_size(&mut self, max: u32) -> &mut Self {
        self.builder.http2().max_header_list_size(max);
        self
    }

    /// 运行服务器。
    pub async fn run<S>(&mut self, service: S) -> Result<(), RunError<L::Error>>
    where
        S: Service<Request> + 'static,
        S::Response: IntoResponse,
        S::Error: Into<BoxError>,
    {
        self.run_with_graceful_shutdown(service, std::future::pending())
            .await
    }

    /// 运行服务器，并设置关机信号用于启动优雅关机。
    pub async fn run_with_graceful_shutdown<S, G>(
        &mut self,
        service: S,
        signal: G,
    ) -> Result<(), RunError<L::Error>>
    where
        S: Service<Request> + 'static,
        S::Response: IntoResponse,
        S::Error: Into<BoxError>,
        G: Future<Output = Option<Duration>> + Send + 'static,
    {
        let mut signal = std::pin::pin!(signal);

        let service = wrapped_service(service);
        let service = hyper::service::service_fn(move |req| {
            let service = service.clone();
            async move {
                let service = service
                    .map_request(compat::into_boluo_request)
                    .map_response(compat::into_hyper_response);
                service.call(req).await
            }
        });

        let graceful = GracefulShutdown::new();

        let timeout = loop {
            tokio::select! {
                timeout = signal.as_mut() => {
                    break timeout;
                }
                incoming = self.listener.accept() => {
                    let (conn, addr) = match incoming {
                        Ok(value) => value,
                        Err(e) => return Err(RunError::Listener(e, graceful)),
                    };

                    let service = service.clone();
                    let service = hyper::service::service_fn(move |mut req| {
                        req.extensions_mut().insert(addr.clone());
                        service.call(req)
                    });

                    let builder = self.builder.clone();
                    let monitor = graceful.monitor();

                    tokio::spawn(async move {
                        let conn = builder.serve_connection_with_upgrades(TokioIo::new(conn), service);
                        let conn = monitor.watch(conn, |conn| conn.graceful_shutdown());
                        conn.await
                    });
                }
            }
        };

        if !graceful.shutdown(timeout).await {
            return Err(RunError::GracefulShutdownTimeout);
        }

        Ok(())
    }
}

fn wrapped_service<S>(service: S) -> ArcService<Request, Response, Infallible>
where
    S: Service<Request> + 'static,
    S::Response: IntoResponse,
    S::Error: Into<BoxError>,
{
    boluo_core::util::__try_downcast(service).unwrap_or_else(|service| {
        let service = service.map_result(|res| {
            res.map_err(Into::into)
                .and_then(|r| r.into_response().map_err(Into::into))
                .or_else(|e| {
                    (StatusCode::INTERNAL_SERVER_ERROR, format!("{e}"))
                        .into_response()
                        .map_err(|e| unreachable!("{e}"))
                })
        });
        service.boxed_arc()
    })
}

/// 服务器运行错误。
#[derive(Debug)]
pub enum RunError<E> {
    /// 优雅关机超时。
    GracefulShutdownTimeout,
    /// 监听器发生错误。
    Listener(E, GracefulShutdown),
}

impl<E: std::fmt::Display> std::fmt::Display for RunError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RunError::GracefulShutdownTimeout => write!(f, "server graceful shutdown timeout"),
            RunError::Listener(e, _) => {
                write!(f, "the server encountered an error while running ({e})")
            }
        }
    }
}

impl<E: std::error::Error> std::error::Error for RunError<E> {}
