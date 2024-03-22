//! HTTP服务器。

mod compat;

mod graceful_shutdown;
pub use graceful_shutdown::GracefulShutdown;

use std::convert::Infallible;
use std::future::Future;
use std::time::Duration;

use boluo_core::http::StatusCode;
use boluo_core::request::Request;
use boluo_core::response::{IntoResponse, Response};
use boluo_core::service::{ArcService, Service, ServiceExt};
use boluo_core::BoxError;
use hyper::service::Service as _;
use hyper_util::rt::{TokioExecutor, TokioIo};
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
{
    /// 使用指定的监听器创建服务器。
    pub fn new(listener: L) -> Self {
        Self {
            listener,
            builder: Builder::new(TokioExecutor::new()),
        }
    }

    /// Set whether HTTP/1 connections should support half-closures.
    ///
    /// Clients can chose to shutdown their write-side while waiting
    /// for the server to respond. Setting this to `true` will
    /// prevent closing the connection immediately if `read`
    /// detects an EOF in the middle of a request.
    ///
    /// Default is `false`.
    #[cfg(feature = "http1")]
    pub fn http1_half_close(&mut self, val: bool) -> &mut Self {
        self.builder.http1().half_close(val);
        self
    }

    /// Enables or disables HTTP/1 keep-alive.
    ///
    /// Default is true.
    #[cfg(feature = "http1")]
    pub fn http1_keep_alive(&mut self, val: bool) -> &mut Self {
        self.builder.http1().keep_alive(val);
        self
    }

    /// Set whether HTTP/1 connections will write header names as title case at
    /// the socket level.
    ///
    /// Note that this setting does not affect HTTP/2.
    ///
    /// Default is false.
    #[cfg(feature = "http1")]
    pub fn http1_title_case_headers(&mut self, enabled: bool) -> &mut Self {
        self.builder.http1().title_case_headers(enabled);
        self
    }

    /// Set whether to support preserving original header cases.
    ///
    /// Currently, this will record the original cases received, and store them
    /// in a private extension on the `Request`. It will also look for and use
    /// such an extension in any provided `Response`.
    ///
    /// Since the relevant extension is still private, there is no way to
    /// interact with the original cases. The only effect this can have now is
    /// to forward the cases in a proxy-like fashion.
    ///
    /// Note that this setting does not affect HTTP/2.
    ///
    /// Default is false.
    #[cfg(feature = "http1")]
    pub fn http1_preserve_header_case(&mut self, enabled: bool) -> &mut Self {
        self.builder.http1().preserve_header_case(enabled);
        self
    }

    /// Set a timeout for reading client request headers. If a client does not
    /// transmit the entire header within this time, the connection is closed.
    ///
    /// Default is None.
    #[cfg(feature = "http1")]
    pub fn http1_header_read_timeout(&mut self, read_timeout: Duration) -> &mut Self {
        self.builder.http1().header_read_timeout(read_timeout);
        self
    }

    /// Set whether HTTP/1 connections should try to use vectored writes,
    /// or always flatten into a single buffer.
    ///
    /// Note that setting this to false may mean more copies of body data,
    /// but may also improve performance when an IO transport doesn't
    /// support vectored writes well, such as most TLS implementations.
    ///
    /// Setting this to true will force hyper to use queued strategy
    /// which may eliminate unnecessary cloning on some TLS backends
    ///
    /// Default is `auto`. In this mode hyper will try to guess which
    /// mode to use
    #[cfg(feature = "http1")]
    pub fn http1_writev(&mut self, val: bool) -> &mut Self {
        self.builder.http1().writev(val);
        self
    }

    /// Set the maximum buffer size for the connection.
    ///
    /// Default is ~400kb.
    ///
    /// # Panics
    ///
    /// The minimum value allowed is 8192. This method panics if the passed `max` is less than the minimum.
    #[cfg(feature = "http1")]
    pub fn http1_max_buf_size(&mut self, max: usize) -> &mut Self {
        self.builder.http1().max_buf_size(max);
        self
    }

    /// Aggregates flushes to better support pipelined responses.
    ///
    /// Experimental, may have bugs.
    ///
    /// Default is false.
    #[cfg(feature = "http1")]
    pub fn http1_pipeline_flush(&mut self, enabled: bool) -> &mut Self {
        self.builder.http1().pipeline_flush(enabled);
        self
    }

    /// Sets the [`SETTINGS_INITIAL_WINDOW_SIZE`][spec] option for HTTP2
    /// stream-level flow control.
    ///
    /// Passing `None` will do nothing.
    ///
    /// If not set, hyper will use a default.
    ///
    /// [spec]: https://http2.github.io/http2-spec/#SETTINGS_INITIAL_WINDOW_SIZE
    #[cfg(feature = "http2")]
    pub fn http2_initial_stream_window_size(&mut self, sz: impl Into<Option<u32>>) -> &mut Self {
        self.builder.http2().initial_stream_window_size(sz);
        self
    }

    /// Sets the max connection-level flow control for HTTP2.
    ///
    /// Passing `None` will do nothing.
    ///
    /// If not set, hyper will use a default.
    #[cfg(feature = "http2")]
    pub fn http2_initial_connection_window_size(
        &mut self,
        sz: impl Into<Option<u32>>,
    ) -> &mut Self {
        self.builder.http2().initial_connection_window_size(sz);
        self
    }

    /// Sets whether to use an adaptive flow control.
    ///
    /// Enabling this will override the limits set in
    /// `http2_initial_stream_window_size` and
    /// `http2_initial_connection_window_size`.
    #[cfg(feature = "http2")]
    pub fn http2_adaptive_window(&mut self, enabled: bool) -> &mut Self {
        self.builder.http2().adaptive_window(enabled);
        self
    }

    /// Sets the maximum frame size to use for HTTP2.
    ///
    /// Passing `None` will do nothing.
    ///
    /// If not set, hyper will use a default.
    #[cfg(feature = "http2")]
    pub fn http2_max_frame_size(&mut self, sz: impl Into<Option<u32>>) -> &mut Self {
        self.builder.http2().max_frame_size(sz);
        self
    }

    /// Sets the [`SETTINGS_MAX_CONCURRENT_STREAMS`][spec] option for HTTP2
    /// connections.
    ///
    /// Default is 200. Passing `None` will remove any limit.
    ///
    /// [spec]: https://http2.github.io/http2-spec/#SETTINGS_MAX_CONCURRENT_STREAMS
    #[cfg(feature = "http2")]
    pub fn http2_max_concurrent_streams(&mut self, max: impl Into<Option<u32>>) -> &mut Self {
        self.builder.http2().max_concurrent_streams(max);
        self
    }

    /// Sets an interval for HTTP2 Ping frames should be sent to keep a
    /// connection alive.
    ///
    /// Pass `None` to disable HTTP2 keep-alive.
    ///
    /// Default is currently disabled.
    ///
    /// # Cargo Feature
    ///
    #[cfg(feature = "http2")]
    pub fn http2_keep_alive_interval(
        &mut self,
        interval: impl Into<Option<Duration>>,
    ) -> &mut Self {
        self.builder.http2().keep_alive_interval(interval);
        self
    }

    /// Sets a timeout for receiving an acknowledgement of the keep-alive ping.
    ///
    /// If the ping is not acknowledged within the timeout, the connection will
    /// be closed. Does nothing if `http2_keep_alive_interval` is disabled.
    ///
    /// Default is 20 seconds.
    ///
    /// # Cargo Feature
    ///
    #[cfg(feature = "http2")]
    pub fn http2_keep_alive_timeout(&mut self, timeout: Duration) -> &mut Self {
        self.builder.http2().keep_alive_timeout(timeout);
        self
    }

    /// Set the maximum write buffer size for each HTTP/2 stream.
    ///
    /// Default is currently ~400KB, but may change.
    ///
    /// # Panics
    ///
    /// The value must be no larger than `u32::MAX`.
    #[cfg(feature = "http2")]
    pub fn http2_max_send_buf_size(&mut self, max: usize) -> &mut Self {
        self.builder.http2().max_send_buf_size(max);
        self
    }

    /// Enables the [extended CONNECT protocol].
    ///
    /// [extended CONNECT protocol]: https://datatracker.ietf.org/doc/html/rfc8441#section-4
    #[cfg(feature = "http2")]
    pub fn http2_enable_connect_protocol(&mut self) -> &mut Self {
        self.builder.http2().enable_connect_protocol();
        self
    }

    /// Sets the max size of received header frames.
    ///
    /// Default is currently ~16MB, but may change.
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
                    let (conn, info) = match incoming {
                        Ok(value) => value,
                        Err(e) => return Err(RunError::Listener(e, graceful)),
                    };

                    let service = service.clone();
                    let service = hyper::service::service_fn(move |mut req| {
                        info.map(|info| req.extensions_mut().insert(info));
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
