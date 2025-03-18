//! 中间件的特征和相关类型的定义。

pub use boluo_core::middleware::*;

mod extension;

pub use extension::{Extension, ExtensionService};
