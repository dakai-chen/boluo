use boluo_core::extract::FromRequest;
use boluo_core::request::Request;

pub use crate::data::Extension;

impl<T> FromRequest for Extension<T>
where
    T: Clone + Send + Sync + 'static,
{
    type Error = ExtensionExtractError;

    async fn from_request(req: &mut Request) -> Result<Self, Self::Error> {
        req.extensions()
            .get::<T>()
            .map(|value| Extension(value.clone()))
            .ok_or_else(|| ExtensionExtractError::MissingExtension {
                name: std::any::type_name::<T>(),
            })
    }
}

/// 扩展提取错误。
#[derive(Debug, Clone, Copy)]
pub enum ExtensionExtractError {
    /// 缺少请求扩展。
    MissingExtension {
        /// 扩展类型名。
        name: &'static str,
    },
}

impl std::fmt::Display for ExtensionExtractError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExtensionExtractError::MissingExtension { name } => {
                write!(f, "missing request extension `{name}`")
            }
        }
    }
}

impl std::error::Error for ExtensionExtractError {}
