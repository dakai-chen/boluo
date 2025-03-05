mod compat;

use std::time::Duration;

use boluo::data::Extension;
use boluo::response::IntoResponse;
use boluo::route::Router;
use boluo::server::Server;
use boluo::service::ServiceExt;
use compat::{CompatTower, CompatTowerHttp};
use tokio::net::TcpListener;
use tower::timeout::TimeoutLayer;
use tower_http::add_extension::AddExtensionLayer;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();

    let app = Router::new()
        .mount(hello)
        .mount(sleep)
        // 避免服务被深度拷贝。
        // 这是因为tower的服务每次调用都需要在内部克隆boluo的服务。
        .boxed_arc()
        .with(CompatTowerHttp(AddExtensionLayer::new("compat tower")))
        .with(CompatTower(TimeoutLayer::new(Duration::from_secs(6))));

    Server::new(listener).run(app).await.unwrap();
}

#[boluo::route("/", method = "GET")]
async fn hello(message: Extension<&'static str>) -> impl IntoResponse {
    message.0
}

#[boluo::route("/sleep", method = "GET")]
async fn sleep() -> impl IntoResponse {
    tokio::time::sleep(Duration::from_secs(10)).await;
}
