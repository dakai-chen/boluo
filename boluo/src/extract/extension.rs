use std::convert::Infallible;

use boluo_core::extract::{FromRequest, OptionalFromRequest};
use boluo_core::request::Request;

pub use crate::data::Extension;

impl<T> FromRequest for Extension<T>
where
    T: Clone + Send + Sync + 'static,
{
    type Error = ExtensionError;

    async fn from_request(req: &mut Request) -> Result<Self, Self::Error> {
        let opt = Option::<Extension<T>>::from_request(req)
            .await
            .map_err(|e| match e {})?;
        opt.ok_or_else(|| ExtensionError::MissingExtension {
            name: std::any::type_name::<T>(),
        })
    }
}

impl<T> OptionalFromRequest for Extension<T>
where
    T: Clone + Send + Sync + 'static,
{
    type Error = Infallible;

    async fn from_request(req: &mut Request) -> Result<Option<Self>, Self::Error> {
        Ok(req
            .extensions()
            .get::<T>()
            .map(|value| Extension(value.clone())))
    }
}

/// 扩展提取错误。
#[derive(Debug, Clone, Copy)]
pub enum ExtensionError {
    /// 缺少请求扩展。
    MissingExtension {
        /// 扩展类型名。
        name: &'static str,
    },
}

impl std::fmt::Display for ExtensionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExtensionError::MissingExtension { name } => {
                write!(f, "missing request extension `{name}`")
            }
        }
    }
}

impl std::error::Error for ExtensionError {}
