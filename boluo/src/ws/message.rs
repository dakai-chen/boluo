use std::borrow::Cow;

use boluo_core::BoxError;
use tokio_tungstenite::tungstenite as ts;

pub use tokio_tungstenite::tungstenite::protocol::{frame::coding::CloseCode, CloseFrame};

/// An enum representing the various forms of a WebSocket message.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Message {
    /// A text WebSocket message
    Text(String),
    /// A binary WebSocket message
    Binary(Vec<u8>),
    /// A ping message with the specified payload
    ///
    /// The payload here must have a length less than 125 bytes
    Ping(Vec<u8>),
    /// A pong message with the specified payload
    ///
    /// The payload here must have a length less than 125 bytes
    Pong(Vec<u8>),
    /// A close message with the optional close frame.
    Close(Option<CloseFrame<'static>>),
}

impl Message {
    pub(super) fn into_tungstenite(self) -> ts::Message {
        match self {
            Self::Text(text) => ts::Message::Text(text),
            Self::Binary(binary) => ts::Message::Binary(binary),
            Self::Ping(ping) => ts::Message::Ping(ping),
            Self::Pong(pong) => ts::Message::Pong(pong),
            Self::Close(close) => ts::Message::Close(close),
        }
    }

    pub(super) fn from_tungstenite(message: ts::Message) -> Message {
        match message {
            ts::Message::Text(text) => Self::Text(text),
            ts::Message::Binary(binary) => Self::Binary(binary),
            ts::Message::Ping(ping) => Self::Ping(ping),
            ts::Message::Pong(pong) => Self::Pong(pong),
            ts::Message::Close(close) => Self::Close(close),
            ts::Message::Frame(_) => unreachable!(),
        }
    }

    pub fn text<S>(text: S) -> Message
    where
        S: Into<String>,
    {
        Message::Text(text.into())
    }

    pub fn binary<D>(data: D) -> Message
    where
        D: Into<Vec<u8>>,
    {
        Message::Binary(data.into())
    }

    pub fn ping<D>(data: D) -> Message
    where
        D: Into<Vec<u8>>,
    {
        Message::Ping(data.into())
    }

    pub fn pong<D>(data: D) -> Message
    where
        D: Into<Vec<u8>>,
    {
        Message::Pong(data.into())
    }

    pub fn close() -> Message {
        Message::Close(None)
    }

    pub fn close_with(code: impl Into<u16>, reason: impl Into<Cow<'static, str>>) -> Message {
        Message::Close(Some(CloseFrame {
            code: CloseCode::from(code.into()),
            reason: reason.into(),
        }))
    }

    pub fn is_text(&self) -> bool {
        matches!(*self, Message::Text(_))
    }

    pub fn is_binary(&self) -> bool {
        matches!(*self, Message::Binary(_))
    }

    pub fn is_ping(&self) -> bool {
        matches!(*self, Message::Ping(_))
    }

    pub fn is_pong(&self) -> bool {
        matches!(*self, Message::Pong(_))
    }

    pub fn is_close(&self) -> bool {
        matches!(*self, Message::Close(_))
    }

    pub fn len(&self) -> usize {
        match self {
            Message::Text(text) => text.len(),
            Message::Binary(data) | Message::Ping(data) | Message::Pong(data) => data.len(),
            Message::Close(data) => data.as_ref().map(|d| d.reason.len()).unwrap_or(0),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn into_bytes(self) -> Vec<u8> {
        match self {
            Message::Text(text) => text.into_bytes(),
            Message::Binary(data) | Message::Ping(data) | Message::Pong(data) => data,
            Message::Close(None) => Vec::new(),
            Message::Close(Some(frame)) => frame.reason.into_owned().into_bytes(),
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Message::Text(text) => text.as_bytes(),
            Message::Binary(data) | Message::Ping(data) | Message::Pong(data) => data,
            Message::Close(None) => &[],
            Message::Close(Some(frame)) => frame.reason.as_bytes(),
        }
    }

    pub fn into_text(self) -> Result<String, BoxError> {
        match self {
            Message::Text(text) => Ok(text),
            Message::Binary(data) | Message::Ping(data) | Message::Pong(data) => {
                String::from_utf8(data).map_err(From::from)
            }
            Message::Close(None) => Ok(String::new()),
            Message::Close(Some(frame)) => Ok(frame.reason.into_owned()),
        }
    }

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
        Message::text(text)
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
        Message::binary(data)
    }
}

impl From<Vec<u8>> for Message {
    fn from(data: Vec<u8>) -> Self {
        Message::binary(data)
    }
}

impl<'a> From<&'a [u8]> for Message {
    fn from(data: &'a [u8]) -> Self {
        Message::binary(data)
    }
}

impl From<Message> for Vec<u8> {
    fn from(message: Message) -> Self {
        message.into_bytes()
    }
}
