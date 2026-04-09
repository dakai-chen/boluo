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
        tokio::signal::ctrl_c().await.ok(); // 等待 ctrl + c 信号
    };

    let result = Server::new(listener)
        .run_with_graceful_shutdown(app, shutdown)
        .await;

    match result {
        Ok(graceful) | Err(RunError::Listener(_, graceful)) => {
            graceful.shutdown(Some(Duration::from_secs(60))).await.ok();
        }
    }
}

#[boluo::route("/", method = "GET")]
async fn hello() -> impl IntoResponse {
    "Hello, World!"
}
