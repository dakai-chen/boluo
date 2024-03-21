use boluo_core::http::{header, HeaderValue};
use boluo_core::response::{IntoResponse, Response};

/// HTML响应。
///
/// 设置响应标头`Content-Type: text/html; charset=utf-8`。
#[derive(Debug, Clone, Copy)]
pub struct Html<T>(pub T);

impl<T> IntoResponse for Html<T>
where
    T: IntoResponse,
{
    type Error = T::Error;

    fn into_response(self) -> Result<Response, Self::Error> {
        let mut res = self.0.into_response()?;
        res.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::TEXT_HTML_UTF_8.as_ref()),
        );
        Ok(res)
    }
}
