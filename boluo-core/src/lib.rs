//! `boluo`的核心类型和特征。

#![forbid(unsafe_code)]
#![warn(
    missing_debug_implementations,
    missing_docs,
    rust_2018_idioms,
    unreachable_pub
)]
#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]

#[macro_use]
mod macros;

#[doc(hidden)]
pub mod util;

pub mod body;
pub mod extract;
pub mod handler;
pub mod middleware;
pub mod request;
pub mod response;
pub mod service;

pub mod http {
    //! [`http`]库的重新导出。

    pub use http::header::{self, HeaderMap, HeaderName, HeaderValue};
    pub use http::method::{self, Method};
    pub use http::status::{self, StatusCode};
    pub use http::uri::{self, Uri};
    pub use http::version::{self, Version};
    pub use http::{Error, Extensions, Result};
}

/// 类型擦除的错误类型别名
pub type BoxError = Box<dyn std::error::Error + Send + Sync>;
