use boluo::middleware::{middleware_fn, Middleware};
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

    let app = Router::new().mount(hello).with(
        log(), // 将日志中间件应用到路由器。
    );

    Server::new(listener).run(app).await.unwrap();
}

#[boluo::route("/", method = "GET")]
async fn hello() -> impl IntoResponse {
    "Hello, World!"
}

/// 简单的日志中间件。
fn log<S>() -> impl Middleware<S, Service = impl HttpService>
where
    S: HttpService<Response = Response, Error = BoxError>,
{
    middleware_fn(|service: S| {
        service
            .map_request(|r: Request| {
                println!(">> {} {}", r.method(), r.uri().path());
                r
            })
            .map_response(|r| {
                println!(":: {}", r.status());
                r
            })
            .map_err(|e| {
                println!("!! {}", e);
                e
            })
    })
}

/// 用于简化特征书写。
trait HttpService<R = Request>:
    Service<R, Response = <Self as HttpService<R>>::Response, Error = <Self as HttpService<R>>::Error>
{
    type Response: IntoResponse;
    type Error: Into<BoxError>;
}

impl<S: ?Sized, R> HttpService<R> for S
where
    S: Service<R>,
    S::Response: IntoResponse,
    S::Error: Into<BoxError>,
{
    type Response = S::Response;
    type Error = S::Error;
}
