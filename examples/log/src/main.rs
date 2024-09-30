use boluo::middleware::simple_middleware_fn;
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

    let app = Router::new().mount(hello).with(simple_middleware_fn(log));

    Server::new(listener).run(app).await.unwrap();
}

#[boluo::route("/", method = "GET")]
async fn hello() -> impl IntoResponse {
    "Hello, World!"
}

/// 简单的日志中间件。
async fn log<S>(req: Request, service: &S) -> Result<Response, BoxError>
where
    S: Service<Request, Response = Response, Error = BoxError>,
{
    println!(">> {} {}", req.method(), req.uri().path());
    let result = service.call(req).await;
    match &result {
        Ok(res) => println!(":: {}", res.status()),
        Err(err) => println!("!! {}", err),
    };
    result
}
