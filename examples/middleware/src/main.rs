use boluo::http::header::AUTHORIZATION;
use boluo::http::HeaderName;
use boluo::middleware::simple_middleware_fn_with_state;
use boluo::request::Request;
use boluo::response::{IntoResponse, Response};
use boluo::route::Router;
use boluo::server::Server;
use boluo::service::{Service, ServiceExt};
use boluo::BoxError;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();

    let app = Router::new()
        .mount(hello)
        // 将中间件挂载到服务上
        .with(simple_middleware_fn_with_state(
            AUTHORIZATION,
            required_request_header,
        ));

    Server::new(listener).run(app).await.unwrap();
}

#[boluo::route("/", method = "GET")]
async fn hello() -> impl IntoResponse {
    "Hello, World!"
}

/// 一个携带状态的中间件。如果指定的请求头不存在，则拒绝此请求。
async fn required_request_header<S>(
    header_name: &HeaderName,
    req: Request,
    service: &S,
) -> Result<Response, BoxError>
where
    S: Service<Request, Response = Response, Error = BoxError>,
{
    if !req.headers().contains_key(header_name) {
        return Err(format!("missing request header `{header_name}`").into());
    }
    service.call(req).await
}
