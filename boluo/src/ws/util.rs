use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use boluo_core::body::Bytes;
use boluo_core::http::header::{HeaderMap, HeaderName, HeaderValue};
use sha1::{Digest, Sha1};

pub(super) fn sign(key: &[u8]) -> HeaderValue {
    let mut sha1 = Sha1::default();
    sha1.update(key);
    sha1.update(&b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11"[..]);
    let b64 = Bytes::from(STANDARD.encode(sha1.finalize()));
    HeaderValue::from_maybe_shared(b64).expect("base64 is a valid value")
}

pub(super) fn header_eq_ignore_case(headers: &HeaderMap, key: HeaderName, value: &str) -> bool {
    if let Some(header) = headers.get(&key) {
        header.as_bytes().eq_ignore_ascii_case(value.as_bytes())
    } else {
        false
    }
}

pub(super) fn header_eq(headers: &HeaderMap, key: HeaderName, value: &str) -> bool {
    if let Some(header) = headers.get(&key) {
        header.as_bytes().eq(value.as_bytes())
    } else {
        false
    }
}
