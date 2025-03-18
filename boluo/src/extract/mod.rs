//! 从请求中提取数据的类型和特征。

pub use boluo_core::extract::*;

mod extension;
mod form;
mod header;
mod json;
mod path;
mod query;

pub use extension::{Extension, ExtensionExtractError};
pub use form::{Form, FormExtractError};
pub use header::{TypedHeader, TypedHeaderExtractError};
pub use json::{Json, JsonExtractError};
pub use path::{Path, PathExtractError, RawPathParams};
pub use query::{Query, QueryExtractError, RawQuery};
