//! 从请求中提取数据的类型和特征。

pub use boluo_core::extract::*;

mod extension;
mod form;
mod header;
mod json;
mod path;
mod query;

pub use extension::{Extension, ExtensionError};
pub use form::{Form, FormError};
pub use header::{TypedHeader, TypedHeaderError};
pub use json::{Json, JsonError};
pub use path::{Path, PathError, RawPathParams};
pub use query::{Query, QueryError, RawQuery};
