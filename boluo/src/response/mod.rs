//! HTTP响应。

pub use boluo_core::response::*;

#[cfg(feature = "sse")]
pub mod sse;

mod extension;
mod form;
mod html;
mod json;
mod redirect;

pub use extension::Extension;
pub use form::{Form, FormResponseError};
pub use html::Html;
pub use json::{Json, JsonResponseError};
pub use redirect::{Redirect, RedirectUriError};
