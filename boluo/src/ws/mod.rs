//! 处理 WebSocket 连接。

mod message;
mod util;

pub use message::{CloseCode, CloseFrame, Message, Utf8Bytes};

use std::pin::Pin;
use std::task::{Context, Poll};

use boluo_core::BoxError;
use boluo_core::body::Body;
use boluo_core::extract::FromRequest;
use boluo_core::http::StatusCode;
use boluo_core::http::header::{self, HeaderValue};
use boluo_core::request::Request;
use boluo_core::response::{IntoResponse, Response};
use boluo_core::upgrade::{OnUpgrade, Upgraded};
use futures_util::{FutureExt, Sink, SinkExt, Stream, StreamExt, ready};
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::tungstenite::protocol::{self, WebSocketConfig};
use tokio_util::compat::{Compat, FuturesAsyncReadCompatExt};

/// 用于建立 WebSocket 连接的提取器。
///
/// # 例子
///
/// ```
/// use boluo::response::IntoResponse;
/// use boluo::ws::{WebSocket, WebSocketUpgrade};
///
/// #[boluo::route("/", method = "GET")]
/// async fn echo(upgrade: WebSocketUpgrade, /* WebSocket 协议升级对象 */) -> impl IntoResponse {
///     upgrade.on_upgrade(handle) // 尝试将 HTTP 协议升级为 WebSocket 协议
/// }
///
/// async fn handle(mut socket: WebSocket) {
///     while let Some(Ok(message)) = socket.recv().await {
///         socket.send(message).await.ok();
///     }
/// }
/// ```
pub struct WebSocketUpgrade {
    config: WebSocketConfig,
    sec_websocket_key: HeaderValue,
    on_upgrade_error: Option<Box<dyn FnOnce(BoxError) + Send>>,
    on_upgrade: OnUpgrade,
}

impl WebSocketUpgrade {
    /// Read buffer capacity. This buffer is eagerly allocated and used for receiving
    /// messages.
    ///
    /// For high read load scenarios a larger buffer, e.g. 128 KiB, improves performance.
    ///
    /// For scenarios where you expect a lot of connections and don't need high read load
    /// performance a smaller buffer, e.g. 4 KiB, would be appropriate to lower total
    /// memory usage.
    ///
    /// The default value is 128 KiB.
    pub fn read_buffer_size(mut self, size: usize) -> Self {
        self.config.read_buffer_size = size;
        self
    }

    /// The target minimum size of the write buffer to reach before writing the data
    /// to the underlying stream.
    /// The default value is 128 KiB.
    ///
    /// If set to `0` each message will be eagerly written to the underlying stream.
    /// It is often more optimal to allow them to buffer a little, hence the default value.
    ///
    /// Note: [`flush`](SinkExt::flush) will always fully write the buffer regardless.
    pub fn write_buffer_size(mut self, size: usize) -> Self {
        self.config.write_buffer_size = size;
        self
    }

    /// The max size of the write buffer in bytes. Setting this can provide backpressure
    /// in the case the write buffer is filling up due to write errors.
    /// The default value is unlimited.
    ///
    /// Note: The write buffer only builds up past [`write_buffer_size`](Self::write_buffer_size)
    /// when writes to the underlying stream are failing. So the **write buffer can not
    /// fill up if you are not observing write errors even if not flushing**.
    ///
    /// Note: Should always be at least [`write_buffer_size + 1 message`](Self::write_buffer_size)
    /// and probably a little more depending on error handling strategy.
    pub fn max_write_buffer_size(mut self, max: usize) -> Self {
        self.config.max_write_buffer_size = max;
        self
    }

    /// The maximum size of an incoming message. `None` means no size limit. The default value is 64 MiB
    /// which should be reasonably big for all normal use-cases but small enough to prevent
    /// memory eating by a malicious user.
    pub fn max_message_size(mut self, max: usize) -> Self {
        self.config.max_message_size = Some(max);
        self
    }

    /// The maximum size of a single incoming message frame. `None` means no size limit. The limit is for
    /// frame payload NOT including the frame header. The default value is 16 MiB which should
    /// be reasonably big for all normal use-cases but small enough to prevent memory eating
    /// by a malicious user.
    pub fn max_frame_size(mut self, max: usize) -> Self {
        self.config.max_frame_size = Some(max);
        self
    }

    /// When set to `true`, the server will accept and handle unmasked frames
    /// from the client. According to the RFC 6455, the server must close the
    /// connection to the client in such cases, however it seems like there are
    /// some popular libraries that are sending unmasked frames, ignoring the RFC.
    /// By default this option is set to `false`, i.e. according to RFC 6455.
    pub fn accept_unmasked_frames(mut self, accept: bool) -> Self {
        self.config.accept_unmasked_frames = accept;
        self
    }

    /// 使用方式请参阅 [`WebSocketUpgrade::on_upgrade`] 函数中的说明。
    pub fn on_upgrade_error<F>(mut self, callback: F) -> Self
    where
        F: FnOnce(BoxError) + Send + 'static,
    {
        self.on_upgrade_error = Some(Box::new(callback));
        self
    }

    /// 尝试将 HTTP 协议升级为 WebSocket 协议，若升级成功则调用提供的异步函数，
    /// 若失败则触发 [`WebSocketUpgrade::on_upgrade_error`] 处理错误。
    ///
    /// # 例子
    ///
    /// ```
    /// use boluo::response::IntoResponse;
    /// use boluo::ws::{WebSocket, WebSocketUpgrade};
    ///
    /// #[boluo::route("/", method = "GET")]
    /// async fn echo(upgrade: WebSocketUpgrade, /* WebSocket 协议升级对象 */) -> impl IntoResponse {
    ///     upgrade
    ///         .on_upgrade_error(|err| println!("{err}")) // 升级失败的错误处理
    ///         .on_upgrade(handle) // 尝试将 HTTP 协议升级为 WebSocket 协议
    /// }
    ///
    /// async fn handle(mut socket: WebSocket) {
    ///     while let Some(Ok(message)) = socket.recv().await {
    ///         socket.send(message).await.ok();
    ///     }
    /// }
    /// ```
    pub fn on_upgrade<F, Fut>(self, callback: F) -> Result<impl IntoResponse, BoxError>
    where
        F: FnOnce(WebSocket) -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let WebSocketUpgrade {
            config,
            sec_websocket_key,
            on_upgrade_error,
            on_upgrade,
        } = self;

        tokio::spawn(async move {
            let socket = match on_upgrade.await {
                Ok(upgraded) => {
                    WebSocket::from_raw_socket(upgraded, protocol::Role::Server, Some(config)).await
                }
                Err(error) => {
                    if let Some(f) = on_upgrade_error {
                        f(error);
                    }
                    return;
                }
            };
            callback(socket).await;
        });

        Response::builder()
            .status(StatusCode::SWITCHING_PROTOCOLS)
            .header(header::CONNECTION, "upgrade")
            .header(header::UPGRADE, "websocket")
            .header(
                header::SEC_WEBSOCKET_ACCEPT,
                util::sign(sec_websocket_key.as_bytes()),
            )
            .body(Body::empty())
            .map_err(From::from)
    }
}

impl std::fmt::Debug for WebSocketUpgrade {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WebSocketUpgrade").finish()
    }
}

impl FromRequest for WebSocketUpgrade {
    type Error = WebSocketUpgradeError;

    async fn from_request(req: &mut Request) -> Result<Self, Self::Error> {
        if !util::header_eq_ignore_case(req.headers(), header::CONNECTION, "upgrade") {
            return Err(WebSocketUpgradeError::InvalidConnectionHeader);
        }
        if !util::header_eq_ignore_case(req.headers(), header::UPGRADE, "websocket") {
            return Err(WebSocketUpgradeError::InvalidUpgradeHeader);
        }
        if !util::header_eq(req.headers(), header::SEC_WEBSOCKET_VERSION, "13") {
            return Err(WebSocketUpgradeError::InvalidSecWebSocketVersionHeader);
        }

        let sec_websocket_key = req
            .headers()
            .get(header::SEC_WEBSOCKET_KEY)
            .cloned()
            .ok_or(WebSocketUpgradeError::MissingSecWebSocketKeyHeader)?;

        let on_upgrade = req
            .extensions_mut()
            .remove::<OnUpgrade>()
            .ok_or(WebSocketUpgradeError::ConnectionNotUpgradable)?;

        Ok(Self {
            config: WebSocketConfig::default(),
            sec_websocket_key,
            on_upgrade_error: None,
            on_upgrade,
        })
    }
}

/// WebSocket 消息流。
pub struct WebSocket {
    inner: WebSocketStream<Compat<Upgraded>>,
}

impl WebSocket {
    async fn from_raw_socket(
        upgraded: Upgraded,
        role: protocol::Role,
        config: Option<WebSocketConfig>,
    ) -> Self {
        WebSocketStream::from_raw_socket(upgraded.compat(), role, config)
            .map(|inner| WebSocket { inner })
            .await
    }

    /// 接收一条消息。
    pub async fn recv(&mut self) -> Option<Result<Message, BoxError>> {
        self.next().await
    }

    /// 发送一条消息。
    pub async fn send(&mut self, msg: Message) -> Result<(), BoxError> {
        self.inner
            .send(msg.into_tungstenite())
            .await
            .map_err(From::from)
    }
}

impl Stream for WebSocket {
    type Item = Result<Message, BoxError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match ready!(Pin::new(&mut self.inner).poll_next(cx)) {
            Some(Ok(msg)) => Poll::Ready(Some(Ok(Message::from_tungstenite(msg)))),
            Some(Err(err)) => Poll::Ready(Some(Err(err.into()))),
            None => Poll::Ready(None),
        }
    }
}

impl Sink<Message> for WebSocket {
    type Error = BoxError;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.inner).poll_ready(cx).map_err(From::from)
    }

    fn start_send(mut self: Pin<&mut Self>, item: Message) -> Result<(), Self::Error> {
        Pin::new(&mut self.inner)
            .start_send(item.into_tungstenite())
            .map_err(From::from)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.inner).poll_flush(cx).map_err(From::from)
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.inner).poll_close(cx).map_err(From::from)
    }
}

impl std::fmt::Debug for WebSocket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WebSocket").finish()
    }
}

/// WebSocket 连接升级失败。
#[derive(Debug, Clone, Copy)]
pub enum WebSocketUpgradeError {
    /// 无效的请求头 `connection`。
    InvalidConnectionHeader,
    /// 无效的请求头 `upgrade`。
    InvalidUpgradeHeader,
    /// 无效的请求头 `sec-websocket-version`。
    InvalidSecWebSocketVersionHeader,
    /// 缺少请求头 `sec-websocket-key`。
    MissingSecWebSocketKeyHeader,
    /// 连接不可升级。
    ConnectionNotUpgradable,
}

impl std::fmt::Display for WebSocketUpgradeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WebSocketUpgradeError::InvalidConnectionHeader => {
                f.write_str("invalid request header `connection`")
            }
            WebSocketUpgradeError::InvalidUpgradeHeader => {
                f.write_str("invalid request header `upgrade`")
            }
            WebSocketUpgradeError::InvalidSecWebSocketVersionHeader => {
                f.write_str("invalid request header `sec-websocket-version`")
            }
            WebSocketUpgradeError::MissingSecWebSocketKeyHeader => {
                f.write_str("missing request header `sec-websocket-key`")
            }
            WebSocketUpgradeError::ConnectionNotUpgradable => {
                f.write_str("connection not upgradable")
            }
        }
    }
}

impl std::error::Error for WebSocketUpgradeError {}
