use std::borrow::Cow;

use boluo_core::BoxError;
use bytes::Bytes;
use tokio_tungstenite::tungstenite as ts;

/// An enum representing the various forms of a WebSocket message.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Message {
    /// A text WebSocket message
    Text(Utf8Bytes),
    /// A binary WebSocket message
    Binary(Bytes),
    /// A ping message with the specified payload
    ///
    /// The payload here must have a length less than 125 bytes
    Ping(Bytes),
    /// A pong message with the specified payload
    ///
    /// The payload here must have a length less than 125 bytes
    Pong(Bytes),
    /// A close message with the optional close frame.
    Close(Option<CloseFrame>),
}

impl Message {
    pub(super) fn into_tungstenite(self) -> ts::Message {
        match self {
            Self::Text(text) => ts::Message::Text(text.into_tungstenite()),
            Self::Binary(binary) => ts::Message::Binary(binary),
            Self::Ping(ping) => ts::Message::Ping(ping),
            Self::Pong(pong) => ts::Message::Pong(pong),
            Self::Close(close) => ts::Message::Close(close.map(CloseFrame::into_tungstenite)),
        }
    }

    pub(super) fn from_tungstenite(message: ts::Message) -> Message {
        match message {
            ts::Message::Text(text) => Self::Text(Utf8Bytes::from_tungstenite(text)),
            ts::Message::Binary(binary) => Self::Binary(binary),
            ts::Message::Ping(ping) => Self::Ping(ping),
            ts::Message::Pong(pong) => Self::Pong(pong),
            ts::Message::Close(close) => Self::Close(close.map(CloseFrame::from_tungstenite)),
            ts::Message::Frame(_) => unreachable!(),
        }
    }

    /// 创建文本消息。
    pub fn text<S>(text: S) -> Message
    where
        S: Into<Utf8Bytes>,
    {
        Message::Text(text.into())
    }

    /// 创建二进制消息。
    pub fn binary<D>(data: D) -> Message
    where
        D: Into<Bytes>,
    {
        Message::Binary(data.into())
    }

    /// 创建 `ping` 消息。
    pub fn ping<D>(data: D) -> Message
    where
        D: Into<Bytes>,
    {
        Message::Ping(data.into())
    }

    /// 创建 `pong` 消息。
    pub fn pong<D>(data: D) -> Message
    where
        D: Into<Bytes>,
    {
        Message::Pong(data.into())
    }

    /// 创建空的关闭消息。
    pub fn close() -> Message {
        Message::Close(None)
    }

    /// 创建带有状态码和原因的关闭消息。
    pub fn close_with(code: impl Into<u16>, reason: impl Into<Utf8Bytes>) -> Message {
        Message::Close(Some(CloseFrame {
            code: CloseCode::from(code.into()),
            reason: reason.into(),
        }))
    }

    /// 判断消息是否为 [`Message::Text`]。
    pub fn is_text(&self) -> bool {
        matches!(*self, Message::Text(_))
    }

    /// 判断消息是否为 [`Message::Binary`]。
    pub fn is_binary(&self) -> bool {
        matches!(*self, Message::Binary(_))
    }

    /// 判断消息是否为 [`Message::Ping`]。
    pub fn is_ping(&self) -> bool {
        matches!(*self, Message::Ping(_))
    }

    /// 判断消息是否为 [`Message::Pong`]。
    pub fn is_pong(&self) -> bool {
        matches!(*self, Message::Pong(_))
    }

    /// 判断消息是否为 [`Message::Close`]。
    pub fn is_close(&self) -> bool {
        matches!(*self, Message::Close(_))
    }

    /// 获取消息的长度。
    pub fn len(&self) -> usize {
        match self {
            Message::Text(text) => text.len(),
            Message::Binary(data) | Message::Ping(data) | Message::Pong(data) => data.len(),
            Message::Close(data) => data.as_ref().map(|d| d.reason.len()).unwrap_or(0),
        }
    }

    /// 判断消息是否为空。
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// 将消息作为二进制数据返回。
    pub fn into_bytes(self) -> Bytes {
        match self {
            Message::Text(text) => text.into(),
            Message::Binary(data) | Message::Ping(data) | Message::Pong(data) => data,
            Message::Close(None) => Bytes::new(),
            Message::Close(Some(frame)) => frame.reason.into(),
        }
    }

    /// 将消息作为二进制数据返回。
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Message::Text(text) => text.as_bytes(),
            Message::Binary(data) | Message::Ping(data) | Message::Pong(data) => data,
            Message::Close(None) => &[],
            Message::Close(Some(frame)) => frame.reason.as_bytes(),
        }
    }

    /// 尝试将消息作为文本数据返回。
    pub fn into_text(self) -> Result<Utf8Bytes, BoxError> {
        match self {
            Message::Text(text) => Ok(text),
            Message::Binary(data) | Message::Ping(data) | Message::Pong(data) => {
                Utf8Bytes::try_from(data).map_err(From::from)
            }
            Message::Close(None) => Ok(Utf8Bytes::default()),
            Message::Close(Some(frame)) => Ok(frame.reason),
        }
    }

    /// 尝试将消息作为文本数据返回。
    pub fn to_text(&self) -> Result<&str, BoxError> {
        match self {
            Message::Text(text) => Ok(text),
            Message::Binary(data) | Message::Ping(data) | Message::Pong(data) => {
                std::str::from_utf8(data).map_err(From::from)
            }
            Message::Close(None) => Ok(""),
            Message::Close(Some(frame)) => Ok(&frame.reason),
        }
    }
}

impl<'a> From<Cow<'a, str>> for Message {
    fn from(text: Cow<'a, str>) -> Self {
        Message::text(text.into_owned())
    }
}

impl From<String> for Message {
    fn from(text: String) -> Self {
        Message::text(text)
    }
}

impl<'a> From<&'a str> for Message {
    fn from(text: &'a str) -> Self {
        Message::text(text)
    }
}

impl<'a> From<Cow<'a, [u8]>> for Message {
    fn from(data: Cow<'a, [u8]>) -> Self {
        Message::binary(data.into_owned())
    }
}

impl From<Vec<u8>> for Message {
    fn from(data: Vec<u8>) -> Self {
        Message::binary(data)
    }
}

impl<'a> From<&'a [u8]> for Message {
    fn from(data: &'a [u8]) -> Self {
        Message::binary(data.to_vec())
    }
}

impl From<Message> for Bytes {
    fn from(message: Message) -> Self {
        message.into_bytes()
    }
}

/// UTF-8 wrapper for [Bytes].
///
/// An [Utf8Bytes] is always guaranteed to contain valid UTF-8.
#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct Utf8Bytes(ts::Utf8Bytes);

impl Utf8Bytes {
    /// Creates from a static str.
    #[inline]
    pub const fn from_static(str: &'static str) -> Self {
        Self(ts::Utf8Bytes::from_static(str))
    }

    /// Returns as a string slice.
    #[inline]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    fn into_tungstenite(self) -> ts::Utf8Bytes {
        self.0
    }

    fn from_tungstenite(data: ts::Utf8Bytes) -> Utf8Bytes {
        Self(data)
    }
}

impl std::fmt::Display for Utf8Bytes {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::ops::Deref for Utf8Bytes {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl core::hash::Hash for Utf8Bytes {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.as_str().hash(state)
    }
}

impl PartialOrd for Utf8Bytes {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Utf8Bytes {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl<T> PartialEq<T> for Utf8Bytes
where
    for<'a> &'a str: PartialEq<T>,
{
    /// ```
    /// use boluo::ws::Utf8Bytes;
    ///
    /// let payload = Utf8Bytes::from_static("foo123");
    /// assert_eq!(payload, "foo123");
    /// assert_eq!(payload, "foo123".to_string());
    /// assert_eq!(payload, &"foo123".to_string());
    /// assert_eq!(payload, std::borrow::Cow::from("foo123"));
    /// ```
    #[inline]
    fn eq(&self, other: &T) -> bool {
        self.as_str() == *other
    }
}

impl From<&str> for Utf8Bytes {
    #[inline]
    fn from(s: &str) -> Self {
        Self(s.into())
    }
}

impl From<String> for Utf8Bytes {
    #[inline]
    fn from(s: String) -> Self {
        Self(s.into())
    }
}

impl From<Cow<'_, str>> for Utf8Bytes {
    #[inline]
    fn from(s: Cow<'_, str>) -> Self {
        match s {
            Cow::Borrowed(s) => s.into(),
            Cow::Owned(s) => s.into(),
        }
    }
}

impl TryFrom<&[u8]> for Utf8Bytes {
    type Error = std::str::Utf8Error;

    #[inline]
    fn try_from(v: &[u8]) -> Result<Self, Self::Error> {
        Ok(std::str::from_utf8(v)?.into())
    }
}

impl TryFrom<Vec<u8>> for Utf8Bytes {
    type Error = std::str::Utf8Error;

    #[inline]
    fn try_from(v: Vec<u8>) -> Result<Self, Self::Error> {
        Bytes::from(v).try_into()
    }
}

impl TryFrom<Cow<'_, [u8]>> for Utf8Bytes {
    type Error = std::str::Utf8Error;

    #[inline]
    fn try_from(v: Cow<'_, [u8]>) -> Result<Self, Self::Error> {
        match v {
            Cow::Borrowed(v) => v.try_into(),
            Cow::Owned(v) => v.try_into(),
        }
    }
}

impl TryFrom<Bytes> for Utf8Bytes {
    type Error = std::str::Utf8Error;

    #[inline]
    fn try_from(bytes: Bytes) -> Result<Self, Self::Error> {
        Ok(Self(bytes.try_into()?))
    }
}

impl From<Utf8Bytes> for Bytes {
    #[inline]
    fn from(Utf8Bytes(bytes): Utf8Bytes) -> Self {
        bytes.into()
    }
}

/// Status code used to indicate why an endpoint is closing the WebSocket connection.
pub type CloseCode = u16;

/// A struct representing the close command.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CloseFrame {
    /// The reason as a code.
    pub code: CloseCode,
    /// The reason as text string.
    pub reason: Utf8Bytes,
}

impl CloseFrame {
    fn into_tungstenite(self) -> ts::protocol::CloseFrame {
        ts::protocol::CloseFrame {
            code: ts::protocol::frame::coding::CloseCode::from(self.code),
            reason: self.reason.into_tungstenite(),
        }
    }

    fn from_tungstenite(data: ts::protocol::CloseFrame) -> CloseFrame {
        CloseFrame {
            code: data.code.into(),
            reason: Utf8Bytes::from_tungstenite(data.reason),
        }
    }
}

impl std::fmt::Display for CloseFrame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.reason, self.code)
    }
}
