use std::any::Any;

use crate::request::Request;
use crate::response::{IntoResponse, Response};
use crate::service::{ArcService, Service, ServiceExt};
use crate::BoxError;

/// Private API
#[doc(hidden)]
pub fn __try_downcast<Src: 'static, Dst: 'static>(src: Src) -> Result<Dst, Src> {
    let mut src = Some(src);
    if let Some(dst) = <dyn Any>::downcast_mut::<Option<Dst>>(&mut src) {
        Ok(dst.take().unwrap())
    } else {
        Err(src.unwrap())
    }
}

/// Private API
#[doc(hidden)]
pub fn __into_arc_service<S>(service: S) -> ArcService<Request, Response, BoxError>
where
    S: Service<Request> + 'static,
    S::Response: IntoResponse,
    S::Error: Into<BoxError>,
{
    __try_downcast(service).unwrap_or_else(|service| {
        let service = service.map_result(|result| {
            result
                .map_err(Into::into)
                .and_then(|r| r.into_response().map_err(Into::into))
        });
        service.boxed_arc()
    })
}

/// 断言`S`是一个[`Service`]。
#[inline]
pub(crate) fn assert_service<S, R>(service: S) -> S
where
    S: Service<R>,
{
    service
}
