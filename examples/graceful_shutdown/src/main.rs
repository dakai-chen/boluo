use std::time::Duration;

use boluo::response::IntoResponse;
use boluo::route::Router;
use boluo::server::{RunError, Server};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();

    let app = Router::new().mount(hello);

    let shutdown = async {
        let _ = tokio::signal::ctrl_c().await; // 等待 ctrl + c 信号
        Some(Duration::from_secs(60))
    };

    let res = Server::new(listener)
        // 根据 ctrl + c 信号优雅的关闭服务器
        .run_with_graceful_shutdown(app, shutdown)
        .await;

    // 如果服务器内部的监听器出错，那么等待服务器内部的请求处理完成
    if let Err(RunError::Listener(_, graceful)) = res {
        graceful.shutdown(Some(Duration::from_secs(60))).await;
    }
}

#[boluo::route("/", method = "GET")]
async fn hello() -> impl IntoResponse {
    "Hello, World!"
}
