use std::convert::Infallible;

use boluo_core::response::{IntoResponseParts, ResponseParts};

pub use crate::data::Extension;

impl<T> IntoResponseParts for Extension<T>
where
    T: Clone + Send + Sync + 'static,
{
    type Error = Infallible;

    fn into_response_parts(self, mut parts: ResponseParts) -> Result<ResponseParts, Self::Error> {
        parts.extensions.insert(self.0);
        Ok(parts)
    }
}
