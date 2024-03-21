use std::convert::Infallible;

use boluo_core::body::Body;
use boluo_core::http::{header::LOCATION, HeaderValue, StatusCode};
use boluo_core::response::{IntoResponse, Response};

/// 将请求重定向到另一个位置的响应。
#[derive(Debug, Clone)]
pub struct Redirect {
    status_code: StatusCode,
    location: HeaderValue,
}

impl Redirect {
    /// Create a new [`Redirect`] that uses a [`303 See Other`][mdn] status code.
    ///
    /// This redirect instructs the client to change the method to GET for the subsequent request
    /// to the given `uri`, which is useful after successful form submission, file upload or when
    /// you generally don't want the redirected-to page to observe the original request method and
    /// body (if non-empty). If you want to preserve the request method and body,
    /// [`Redirect::temporary`] should be used instead.
    ///
    /// # Panics
    ///
    /// If `uri` isn't a valid [`HeaderValue`].
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/303
    pub fn to(uri: &str) -> Result<Self, RedirectUriError> {
        Self::with_status_code(StatusCode::SEE_OTHER, uri)
    }

    /// Create a new [`Redirect`] that uses a [`307 Temporary Redirect`][mdn] status code.
    ///
    /// This has the same behavior as [`Redirect::to`], except it will preserve the original HTTP
    /// method and body.
    ///
    /// # Panics
    ///
    /// If `uri` isn't a valid [`HeaderValue`].
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/307
    pub fn temporary(uri: &str) -> Result<Self, RedirectUriError> {
        Self::with_status_code(StatusCode::TEMPORARY_REDIRECT, uri)
    }

    /// Create a new [`Redirect`] that uses a [`308 Permanent Redirect`][mdn] status code.
    ///
    /// # Panics
    ///
    /// If `uri` isn't a valid [`HeaderValue`].
    ///
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/308
    pub fn permanent(uri: &str) -> Result<Self, RedirectUriError> {
        Self::with_status_code(StatusCode::PERMANENT_REDIRECT, uri)
    }

    // This is intentionally not public since other kinds of redirects might not
    // use the `Location` header, namely `304 Not Modified`.
    fn with_status_code(status_code: StatusCode, uri: &str) -> Result<Self, RedirectUriError> {
        debug_assert!(
            status_code.is_redirection(),
            "not a redirection status code"
        );

        HeaderValue::try_from(uri)
            .map_err(|_| RedirectUriError(uri.to_owned()))
            .map(|location| Self {
                status_code,
                location,
            })
    }
}

impl IntoResponse for Redirect {
    type Error = Infallible;

    fn into_response(self) -> Result<Response, Self::Error> {
        Response::builder()
            .status(self.status_code)
            .header(LOCATION, self.location)
            .body(Body::empty())
            .map_err(|e| unreachable!("{e}"))
    }
}

/// 重定向的URI不是有效的标头值。
#[derive(Debug, Clone)]
pub struct RedirectUriError(pub String);

impl std::fmt::Display for RedirectUriError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "redirect uri isn't a valid header value ({})", self.0)
    }
}

impl std::error::Error for RedirectUriError {}
