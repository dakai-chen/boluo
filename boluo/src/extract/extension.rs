use boluo_core::extract::FromRequest;
use boluo_core::request::Request;

pub use crate::data::Extension;

impl<T> FromRequest for Extension<T>
where
    T: Clone + Send + Sync + 'static,
{
    type Error = ExtractExtensionError;

    async fn from_request(req: &mut Request) -> Result<Self, Self::Error> {
        req.extensions()
            .get::<T>()
            .map(|value| Extension(value.clone()))
            .ok_or_else(|| ExtractExtensionError(std::any::type_name::<T>()))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ExtractExtensionError(&'static str);

impl std::fmt::Display for ExtractExtensionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "extension does not exist (`{}`)", self.0)
    }
}

impl std::error::Error for ExtractExtensionError {}
