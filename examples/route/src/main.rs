use boluo::handler::handler_fn;
use boluo::request::Request;
use boluo::response::IntoResponse;
use boluo::route::Router;
use boluo::server::Server;
use boluo::service::Service;
use boluo::BoxError;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    Server::new(TcpListener::bind("127.0.0.1:3000").await.unwrap())
        .run(app())
        .await
        .unwrap();
}

fn app() -> impl Service<Request, Response = impl IntoResponse, Error = impl Into<BoxError>> {
    let ab = Router::new()
        .route("/a", handler_fn(|| async { "a" }))
        .route("/b", handler_fn(|| async { "b" }));

    let cd = Router::new()
        .route("/c", handler_fn(|| async { "c" }))
        .route("/d", handler_fn(|| async { "d" }));

    Router::new()
        // 路由
        .route("/a", handler_fn(|| async { "a" }))
        .route("/b", handler_fn(|| async { "b" }))
        // 嵌套路由
        .scope("/x", ab)
        // 将其他路由器的路由合并到当前路由器
        .merge(cd)
        // 挂载宏定义路由
        .mount(f)
}

#[boluo::route("/f", method = "GET")]
async fn f() -> impl IntoResponse {
    "f"
}
