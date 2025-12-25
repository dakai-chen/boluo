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

    async fn call(&self, request: HyperRequest<Incoming>) -> Result<Self::Response, Self::Error> {
        self.service
            .call(request_from_hyper(request))
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
            result
                .into_response()
                .or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
        });
        service.boxed_arc()
    })
}

fn request_from_hyper(request: HyperRequest<Incoming>) -> Request {
    let (parts, body) = request.into_parts();
    let mut request = Request::new(Body::new(body));
    *request.method_mut() = parts.method;
    *request.uri_mut() = parts.uri;
    *request.version_mut() = parts.version;
    *request.headers_mut() = parts.headers;
    *request.extensions_mut() = replace_hyper_upgrade(parts.extensions);
    request
}

fn response_to_hyper(response: Response) -> HyperResponse<Body> {
    let (parts, body) = response.into_inner();
    let mut response = HyperResponse::new(body);
    *response.status_mut() = parts.status;
    *response.version_mut() = parts.version;
    *response.headers_mut() = parts.headers;
    *response.extensions_mut() = parts.extensions;
    response
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
