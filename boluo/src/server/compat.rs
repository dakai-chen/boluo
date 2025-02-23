use boluo_core::body::Body;
use boluo_core::http::Extensions;
use boluo_core::request::Request;
use boluo_core::response::Response;
use boluo_core::upgrade::{OnUpgrade, Upgraded};
use hyper::Request as HyperRequest;
use hyper::Response as HyperResponse;
use hyper::body::Incoming;
use hyper::upgrade::OnUpgrade as HyperOnUpgrade;
use hyper_util::rt::TokioIo;
use tokio_util::compat::TokioAsyncReadCompatExt;

pub(super) fn request_into_boluo(req: HyperRequest<Incoming>) -> Request {
    let (parts, body) = req.into_parts();
    let mut req = Request::new(Body::new(body));
    *req.method_mut() = parts.method;
    *req.uri_mut() = parts.uri;
    *req.version_mut() = parts.version;
    *req.headers_mut() = parts.headers;
    *req.extensions_mut() = replace_upgrade(parts.extensions);
    req
}

pub(super) fn response_into_hyper(res: Response) -> HyperResponse<Body> {
    let (parts, body) = res.into_inner();
    let mut res = HyperResponse::new(body);
    *res.status_mut() = parts.status;
    *res.version_mut() = parts.version;
    *res.headers_mut() = parts.headers;
    *res.extensions_mut() = parts.extensions;
    res
}

fn upgrade_into_boluo(on_upgrade: HyperOnUpgrade) -> OnUpgrade {
    OnUpgrade::new(async {
        on_upgrade
            .await
            .map(|upgraded| Upgraded::new(TokioIo::new(upgraded).compat()))
            .map_err(Into::into)
    })
}

fn replace_upgrade(mut extensions: Extensions) -> Extensions {
    if let Some(on_upgrade) = extensions.remove::<HyperOnUpgrade>() {
        extensions.insert(upgrade_into_boluo(on_upgrade));
    }
    extensions
}
