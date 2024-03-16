use boluo_core::body::Body;
use boluo_core::request::Request;
use boluo_core::response::Response;
use hyper::body::Incoming;

pub(super) fn into_boluo_request(req: hyper::Request<Incoming>) -> Request {
    let (parts, body) = req.into_parts();
    let mut req = Request::new(Body::new(body));
    *req.method_mut() = parts.method;
    *req.uri_mut() = parts.uri;
    *req.version_mut() = parts.version;
    *req.headers_mut() = parts.headers;
    *req.extensions_mut() = parts.extensions;
    req
}

pub(super) fn into_hyper_response(res: Response) -> hyper::Response<Body> {
    let (parts, body) = res.into_inner();
    let mut res = hyper::Response::new(body);
    *res.status_mut() = parts.status;
    *res.version_mut() = parts.version;
    *res.headers_mut() = parts.headers;
    *res.extensions_mut() = parts.extensions;
    res
}
