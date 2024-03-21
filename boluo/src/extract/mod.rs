//! 从请求中提取数据的类型和特征。

pub use boluo_core::extract::*;

mod extension;
pub use extension::{Extension, ExtensionExtractError};

mod form;
pub use form::{Form, FormExtractError};

mod query;
pub use query::{Query, QueryExtractError, RawQuery};

mod json;
pub use json::{Json, JsonExtractError};

mod path;
pub use path::{Path, PathExtractError, RawPathParams};

mod header;
pub use header::{
    HeaderOfName, HeaderOfNameExtractError, OptionalHeaderOfName, OptionalRawHeaderOfName,
    RawHeaderOfName,
};
