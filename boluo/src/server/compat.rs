use std::convert::Infallible;

use boluo_core::BoxError;
use boluo_core::body::Body;
use boluo_core::http::{Extensions, StatusCode};
use boluo_core::request::Request;
use boluo_core::response::{IntoResponse, Response};
use boluo_core::service::{ArcService, Service, ServiceExt};
use boluo_core::upgrade::{OnUpgrade, Upgraded};
use hyper::Request as HyperRequest;
use hyper::Response as HyperResponse;
use hyper::body::Incoming;
use hyper::upgrade::OnUpgrade as HyperOnUpgrade;
use hyper_util::rt::TokioIo;
use tokio_util::compat::TokioAsyncReadCompatExt;

#[derive(Clone)]
pub(super) struct ServiceToHyper {
    service: ArcService<Request, Response, Infallible>,
}

impl Service<HyperRequest<Incoming>> for ServiceToHyper {
    type Response = HyperResponse<Body>;
    type Error = Infallible;

    async fn call(&self, req: HyperRequest<Incoming>) -> Result<Self::Response, Self::Error> {
        self.service
            .call(request_from_hyper(req))
            .await
            .map(response_to_hyper)
    }
}

pub(super) fn service_to_hyper<S>(service: S) -> ServiceToHyper
where
    S: Service<Request> + 'static,
    S::Response: IntoResponse,
    S::Error: Into<BoxError>,
{
    ServiceToHyper {
        service: into_arc_service(service),
    }
}

fn into_arc_service<S>(service: S) -> ArcService<Request, Response, Infallible>
where
    S: Service<Request> + 'static,
    S::Response: IntoResponse,
    S::Error: Into<BoxError>,
{
    boluo_core::util::__try_downcast(service).unwrap_or_else(|service| {
        let service = service.map_result(|result| {
            result.into_response().or_else(|e| {
                (StatusCode::INTERNAL_SERVER_ERROR, format!("{e}"))
                    .into_response()
                    .map_err(|e| unreachable!("{e}"))
            })
        });
        service.boxed_arc()
    })
}

fn request_from_hyper(req: HyperRequest<Incoming>) -> Request {
    let (parts, body) = req.into_parts();
    let mut req = Request::new(Body::new(body));
    *req.method_mut() = parts.method;
    *req.uri_mut() = parts.uri;
    *req.version_mut() = parts.version;
    *req.headers_mut() = parts.headers;
    *req.extensions_mut() = replace_hyper_upgrade(parts.extensions);
    req
}

fn response_to_hyper(res: Response) -> HyperResponse<Body> {
    let (parts, body) = res.into_inner();
    let mut res = HyperResponse::new(body);
    *res.status_mut() = parts.status;
    *res.version_mut() = parts.version;
    *res.headers_mut() = parts.headers;
    *res.extensions_mut() = parts.extensions;
    res
}

fn upgrade_from_hyper(on_upgrade: HyperOnUpgrade) -> OnUpgrade {
    OnUpgrade::new(async {
        on_upgrade
            .await
            .map(|upgraded| Upgraded::new(TokioIo::new(upgraded).compat()))
            .map_err(Into::into)
    })
}

fn replace_hyper_upgrade(mut extensions: Extensions) -> Extensions {
    if let Some(on_upgrade) = extensions.remove::<HyperOnUpgrade>() {
        extensions.insert(upgrade_from_hyper(on_upgrade));
    }
    extensions
}
