#![forbid(unsafe_code)]
#![deny(unreachable_pub)]
#![warn(missing_debug_implementations)]
#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]

pub use boluo_core::name;
pub use boluo_core::BoxError;
pub use boluo_core::{body, handler, http, request, service};

pub use boluo_macros::route;

pub mod data;
pub mod extract;
pub mod middleware;
pub mod response;
pub mod route;

#[cfg(all(feature = "server", any(feature = "http1", feature = "http2")))]
pub mod server;

#[cfg(feature = "listener")]
pub mod listener;

#[cfg(feature = "multipart")]
pub mod multipart;

#[cfg(feature = "ws")]
pub mod ws;

#[cfg(feature = "fs")]
pub mod fs;
